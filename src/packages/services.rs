//! Reviewed local service profiles administered only through Mirket’s private Podman engine.
use super::{Package, Registry};
use crate::{paths::Paths, podman::Engine, util};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Service {
    pub profile: String,
    /// Platform-specific OCI manifests, not mutable tags or multi-platform indices.
    pub images: BTreeMap<String, String>,
}
const ALIASES: [&str; 5] = [
    "penpot-postgres",
    "penpot-valkey",
    "penpot-backend",
    "penpot-exporter",
    "penpot-frontend",
];
const LABEL: &str = "org.mirket.service-owner";

pub(super) fn validate(service: &Service) -> Result<()> {
    if service.profile != "penpot"
        || service
            .images
            .keys()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>()
            != ALIASES.into_iter().collect()
    {
        bail!("unknown or incomplete reviewed local service profile");
    }
    for (name, image) in &service.images {
        let repo = match name.as_str() {
            "penpot-postgres" => "postgres",
            "penpot-valkey" => "valkey/valkey",
            "penpot-backend" => "penpotapp/backend",
            "penpot-exporter" => "penpotapp/exporter",
            "penpot-frontend" => "penpotapp/frontend",
            _ => unreachable!(),
        };
        let digest = image
            .strip_prefix(&format!("{repo}@sha256:"))
            .context("service requires a reviewed repository and SHA-256 manifest")?;
        if digest.len() != 64
            || !digest
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            bail!("invalid OCI digest");
        }
    }
    Ok(())
}

fn run(engine: &Engine, args: impl IntoIterator<Item = String>) -> Result<String> {
    engine.run(
        &args.into_iter().collect::<Vec<_>>(),
        Duration::from_secs(30),
    )
}
fn owner(paths: &Paths) -> String {
    format!(
        "mirket-penpot-{}",
        &util::sha256(paths.root.to_string_lossy().as_bytes())[..16]
    )
}
fn data(paths: &Paths) -> PathBuf {
    paths.root.join("packages/data/penpot/service")
}
fn executable(paths: &Paths) -> Result<Engine> {
    let engine = Engine::resolve(paths)?;
    let info = engine.ready()?;
    if info["host"]["arch"] != "arm64" {
        bail!("Penpot currently requires a verified native Linux ARM64 Podman engine");
    }
    Ok(engine)
}
fn qualified_image(image: &str) -> String {
    if image.starts_with("postgres@") {
        format!("docker.io/library/{image}")
    } else {
        format!("docker.io/{image}")
    }
}
fn verify_image(binary: &Engine, image: &str) -> Result<()> {
    let value: Value = serde_json::from_str(&run(
        binary,
        [
            "image".into(),
            "inspect".into(),
            qualified_image(image),
            "--format".into(),
            "{{json .}}".into(),
        ],
    )?)?;
    if value["Architecture"] != "arm64"
        || value["Os"] != "linux"
        || value["RepoDigests"].as_array().is_none_or(|v| {
            !v.iter()
                .any(|d| d.as_str() == Some(qualified_image(image).as_str()))
        })
    {
        bail!("local OCI image does not match the pinned Linux ARM64 manifest: {image}");
    }
    Ok(())
}
pub(super) fn prepare(paths: &Paths, package: &Package) -> Result<()> {
    let Some(service) = &package.service else {
        return Ok(());
    };
    validate(service)?;
    let bin = executable(paths)?;
    for image in service.images.values() {
        if verify_image(&bin, image).is_err() {
            eprintln!("mirket package: pulling pinned local service image {image}");
            bin.run(
                &[
                    "pull".into(),
                    "--quiet".into(),
                    "--platform".into(),
                    "linux/arm64".into(),
                    qualified_image(image),
                ],
                Duration::from_secs(600),
            )?;
            verify_image(&bin, image)?;
        }
    }
    Ok(())
}
pub(super) fn verify(paths: &Paths, package: &Package) -> Result<()> {
    let Some(service) = &package.service else {
        return Ok(());
    };
    let bin = executable(paths)?;
    for image in service.images.values() {
        verify_image(&bin, image)?;
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[derive(Debug)]
struct State {
    schema_version: u32,
    owner: String,
    port: u16,
    #[serde(default)]
    engine_owner: Option<String>,
    database_password: String,
    service_secret: String,
    images: BTreeMap<String, String>,
}
fn state(paths: &Paths) -> Result<State> {
    let value: State = serde_json::from_slice(
        &util::read_bounded(&data(paths).join("state.json"), 16384)
            .context("Penpot service is not configured; run mirket package start penpot")?,
    )?;
    if value.schema_version != 1
        || value.owner != owner(paths)
        || value.port <= 1024
        || value.database_password.len() != 64
        || value.service_secret.len() != 128
        || !value
            .database_password
            .bytes()
            .chain(value.service_secret.bytes())
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        bail!("invalid private Penpot service state");
    }
    validate(&Service {
        profile: "penpot".into(),
        images: value.images.clone(),
    })?;
    if value.engine_owner.is_none() {
        bail!(
            "Existing Penpot data is not bound to this Podman environment. Preserve its service data and export portable designs with the runtime that created them before provisioning a separate environment; Mirket will not replace or delete it."
        );
    }
    Ok(value)
}
fn bound_state(paths: &Paths, engine: &Engine) -> Result<State> {
    let value = state(paths)?;
    if value.engine_owner.as_deref() != Some(engine.owner().as_str()) {
        bail!(
            "Penpot service data belongs to another Podman environment; preserve its data and use its owning environment or portable design exports"
        );
    }
    Ok(value)
}
fn secret() -> Result<String> {
    let mut bytes = [0; 32];
    fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
fn save_state(paths: &Paths, state: &State) -> Result<()> {
    let dir = data(paths);
    util::ensure_directory(&dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))?;
    }
    util::atomic_write(&dir.join("state.json"), &serde_json::to_vec_pretty(state)?)
}
fn inspected(binary: &Engine, kind: &str, name: &str) -> Result<Value> {
    serde_json::from_str(&run(
        binary,
        [
            kind.into(),
            "inspect".into(),
            name.into(),
            "--format".into(),
            "{{json .}}".into(),
        ],
    )?)
    .context("invalid Podman ownership metadata")
}
fn own_resource(binary: &Engine, kind: &str, name: &str, own: &str) -> Result<Option<Value>> {
    // List first distinguishes absence from a failing daemon or invalid inspect response.
    let listed = run(
        binary,
        match kind {
            "container" => vec![
                "container".into(),
                "ls".into(),
                "--all".into(),
                "--format".into(),
                "{{.Names}}".into(),
            ],
            "network" => vec![
                "network".into(),
                "ls".into(),
                "--format".into(),
                "{{.Name}}".into(),
            ],
            _ => vec![
                "volume".into(),
                "ls".into(),
                "--format".into(),
                "{{.Name}}".into(),
            ],
        },
    )?;
    if !listed.lines().any(|line| line == name) {
        return Ok(None);
    }
    let value = inspected(binary, kind, name)?;
    let labels = if kind == "container" {
        &value["Config"]["Labels"]
    } else if kind == "network" {
        &value["labels"]
    } else {
        &value["Labels"]
    };
    if labels[LABEL] != own {
        bail!("refusing unrelated Podman {kind}: {name}");
    }
    Ok(Some(value))
}
fn container(own: &str, alias: &str) -> String {
    format!("{own}-{alias}")
}
fn url(state: &State) -> String {
    format!("http://127.0.0.1:{}", state.port)
}
const FLAGS: &str = "disable-email-verification disable-secure-session-cookies enable-registration enable-login-with-password disable-telemetry enable-air-gapped-conf disable-google-fonts-provider disable-dashboard-templates-section";
// The pinned frontend's entrypoint modifies immutable application/configuration files.
// This fixed startup uses its reviewed templates, preserving them on the read-only root.
// No caller can change this script, configuration, command or environment contract.
const FRONTEND_START: &str = r#"set -euo pipefail
mkdir -p /run/mirket/home /run/mirket/tmp /run/mirket/nginx
cp -R /etc/nginx/. /run/mirket/nginx/
rm -f /run/mirket/nginx/overrides/location.d/external-locations.conf /run/mirket/nginx/overrides/server.d/admin-console-locations.conf /run/mirket/nginx/overrides/server.d/mcp-locations.conf
sed -e "s|^//var penpotFlags = .*;|var penpotFlags = \"$PENPOT_FLAGS\";|g" /var/www/app/js/config.js > /run/mirket/config.js
printf 'var penpotPublicURI = "%s";\n' "$PENPOT_PUBLIC_URI" >> /run/mirket/config.js
export PENPOT_BACKEND_URI=http://penpot-backend:6060 PENPOT_EXPORTER_URI=http://penpot-exporter:6061 PENPOT_HTTP_SERVER_MAX_BODY_SIZE=67108864 PENPOT_IPV6_LISTEN_DIRECTIVE=""
envsubst '$PENPOT_BACKEND_URI,$PENPOT_EXPORTER_URI,$PENPOT_HTTP_SERVER_MAX_BODY_SIZE,$PENPOT_IPV6_LISTEN_DIRECTIVE' < /tmp/nginx.conf.template | sed -e 's|/etc/nginx/|/run/mirket/nginx/|g' -e 's|/tmp/|/run/mirket/tmp/|g' -e '/location = \/js\/config.js {/a\                alias /run/mirket/config.js;' > /run/mirket/nginx/nginx.conf
export PENPOT_INTERNAL_RESOLVER="$(awk 'BEGIN{ORS=" "} $1=="nameserver" { sub(/%.*$/,"",$2); print ($2 ~ ":")? "["$2"]": $2}' /etc/resolv.conf)"
envsubst '$PENPOT_INTERNAL_RESOLVER' < /tmp/resolvers.conf.template > /run/mirket/nginx/overrides/http.d/resolvers.conf
exec nginx -c /run/mirket/nginx/nginx.conf -g 'daemon off;'
"#;
#[derive(Clone, Copy)]
struct Policy {
    uid: u32,
    memory_mib: u64,
    cpus: u64,
    pids: u64,
    tmp_mib: u64,
}
fn policy(alias: &str) -> Policy {
    match alias {
        "penpot-postgres" => Policy {
            uid: 999,
            memory_mib: 512,
            cpus: 1,
            pids: 128,
            tmp_mib: 32,
        },
        "penpot-valkey" => Policy {
            uid: 999,
            memory_mib: 256,
            cpus: 1,
            pids: 128,
            tmp_mib: 16,
        },
        "penpot-backend" => Policy {
            uid: 1001,
            memory_mib: 2048,
            cpus: 2,
            pids: 512,
            tmp_mib: 128,
        },
        "penpot-exporter" => Policy {
            uid: 1001,
            memory_mib: 1024,
            cpus: 2,
            pids: 512,
            tmp_mib: 128,
        },
        "penpot-frontend" => Policy {
            uid: 1001,
            memory_mib: 256,
            cpus: 1,
            pids: 128,
            tmp_mib: 64,
        },
        _ => unreachable!("validated service alias"),
    }
}
fn tmpfs(alias: &str) -> BTreeMap<String, String> {
    let p = policy(alias);
    let mut mounts = BTreeMap::from([(
        "/run/mirket".into(),
        format!("rw,noexec,nosuid,nodev,size={}m,mode=1777", p.tmp_mib),
    )]);
    if alias == "penpot-postgres" {
        mounts.insert(
            "/run/postgresql".into(),
            "rw,noexec,nosuid,nodev,size=8m,mode=1777".into(),
        );
    }
    if alias == "penpot-backend" {
        // This reviewed backend hard-codes its storage temp directory.
        mounts.insert(
            "/tmp/penpot".into(),
            "rw,noexec,nosuid,nodev,size=64m,mode=1777".into(),
        );
        // The pinned HTTP multipart parser uses this fixed upload directory.
        mounts.insert(
            "/tmp/undertow".into(),
            "rw,noexec,nosuid,nodev,size=64m,mode=1777".into(),
        );
        // The approved JAR extracts its pinned zstd JNI library before loading.
        // Keep executable mappings confined to this private bounded directory.
        mounts.insert(
            "/run/mirket-native".into(),
            "rw,exec,nosuid,nodev,size=8m,mode=1777".into(),
        );
    }
    mounts
}
fn service_env(state: &State, alias: &str) -> BTreeMap<&'static str, String> {
    let mut environment = match alias {
        "penpot-postgres" => BTreeMap::from([
            ("POSTGRES_INITDB_ARGS", "--data-checksums".into()),
            ("POSTGRES_DB", "penpot".into()),
            ("POSTGRES_USER", "penpot".into()),
            ("POSTGRES_PASSWORD", state.database_password.clone()),
        ]),
        "penpot-valkey" => BTreeMap::new(),
        "penpot-backend" => BTreeMap::from([
            ("PENPOT_FLAGS", FLAGS.into()),
            ("PENPOT_PUBLIC_URI", url(state)),
            ("PENPOT_SECRET_KEY", state.service_secret.clone()),
            (
                "PENPOT_DATABASE_URI",
                "postgresql://penpot-postgres/penpot".into(),
            ),
            ("PENPOT_DATABASE_USERNAME", "penpot".into()),
            ("PENPOT_DATABASE_PASSWORD", state.database_password.clone()),
            ("PENPOT_DATABASE_MIN_POOL_SIZE","4".into()),
            ("PENPOT_DATABASE_MAX_POOL_SIZE","16".into()),
            ("PENPOT_REDIS_URI", "redis://penpot-valkey/0".into()),
            ("PENPOT_TELEMETRY_ENABLED", "false".into()),
            ("PENPOT_OBJECTS_STORAGE_BACKEND", "fs".into()),
            (
                "PENPOT_OBJECTS_STORAGE_FS_DIRECTORY",
                "/opt/data/assets".into(),
            ),
            (
                "JAVA_OPTS",
                "-Xms128m -Xmx768m -XX:TieredStopAtLevel=1 -Djava.io.tmpdir=/run/mirket -Duser.home=/run/mirket -DZstdTempFolder=/run/mirket-native".into(),
            ),
        ]),
        _ => BTreeMap::from([
            ("PENPOT_FLAGS", FLAGS.into()),
            ("PENPOT_PUBLIC_URI", url(state)),
            ("PENPOT_INTERNAL_URI", "http://penpot-frontend:8080".into()),
        ]),
    };
    if alias == "penpot-exporter" {
        environment.extend([
            ("PENPOT_SECRET_KEY", state.service_secret.clone()),
            ("PENPOT_REDIS_URI", "redis://penpot-valkey/0".into()),
            ("PENPOT_TEMPDIR", "/run/mirket/penpot".into()),
            ("PENPOT_BROWSER_POOL_MIN", "0".into()),
            ("PENPOT_BROWSER_POOL_MAX", "1".into()),
        ]);
    }
    environment.extend([
        ("HOME", "/run/mirket".into()),
        ("TMPDIR", "/run/mirket".into()),
        ("XDG_CACHE_HOME", "/run/mirket/cache".into()),
        ("container", "podman".into()),
        ("HOSTNAME", alias.into()),
    ]);
    for key in [
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "FTP_PROXY",
        "NO_PROXY",
        "ALL_PROXY",
        "http_proxy",
        "https_proxy",
        "ftp_proxy",
        "no_proxy",
        "all_proxy",
    ] {
        environment.insert(key, String::new());
    }
    environment
}
fn expected_command(image: &Value, alias: &str) -> Value {
    match alias {
        "penpot-frontend" => json!(["-c", FRONTEND_START]),
        "penpot-postgres" => json!([
            "postgres",
            "-c",
            "max_connections=40",
            "-c",
            "shared_buffers=64MB"
        ]),
        "penpot-valkey" => json!([
            "valkey-server",
            "--save",
            "60",
            "1",
            "--appendonly",
            "yes",
            "--dir",
            "/data",
            "--maxmemory",
            "128mb",
            "--maxmemory-policy",
            "noeviction"
        ]),
        _ => image["Config"]["Cmd"].clone(),
    }
}
fn expected_entrypoint(image: &Value, alias: &str) -> Value {
    if alias == "penpot-frontend" {
        json!(["/bin/bash"])
    } else if image["Config"]["Entrypoint"].is_null() {
        json!([])
    } else {
        image["Config"]["Entrypoint"].clone()
    }
}
fn config_string(value: &Value) -> Result<&str> {
    // OCI configs omit empty string fields; container inspect materializes them.
    match value {
        Value::Null => Ok(""),
        Value::String(text) => Ok(text),
        _ => bail!("invalid Podman string configuration"),
    }
}
fn validate_container(value: &Value, image: &Value, state: &State, alias: &str) -> Result<()> {
    let p = policy(alias);
    let working_dir = config_string(&image["Config"]["WorkingDir"])?;
    let stop_signal = config_string(&image["Config"]["StopSignal"])?;
    if value["Config"]["Image"] != qualified_image(&state.images[alias])
        || value["Image"] != image["Id"]
        || value["Config"]["Cmd"] != expected_command(image, alias)
        || value["Config"]["Entrypoint"] != expected_entrypoint(image, alias)
        || value["Config"]["User"] != format!("{}:{}", p.uid, p.uid)
        || config_string(&value["Config"]["WorkingDir"])?
            != if working_dir.is_empty() {
                "/"
            } else {
                working_dir
            }
        || value["Config"]["Healthcheck"] != image["Config"]["Healthcheck"]
        || config_string(&value["Config"]["StopSignal"])?
            != if stop_signal.is_empty() {
                "SIGTERM"
            } else {
                stop_signal
            }
        || value["Config"]["Hostname"] != alias
    {
        bail!("Penpot container executable/image configuration has drifted: {alias}");
    }
    let parse_env = |value: &Value| -> Result<BTreeMap<String, String>> {
        let entries = value.as_array().context("invalid container environment")?;
        let mut result = BTreeMap::new();
        for entry in entries {
            let (key, val) = entry
                .as_str()
                .and_then(|e| e.split_once('='))
                .context("invalid container environment entry")?;
            if result.insert(key.into(), val.into()).is_some() {
                bail!("duplicate container environment key");
            }
        }
        Ok(result)
    };
    let mut expected = parse_env(&image["Config"]["Env"])?;
    for (key, val) in service_env(state, alias) {
        expected.insert(key.into(), val);
    }
    if parse_env(&value["Config"]["Env"])? != expected {
        bail!("Penpot container environment has drifted: {alias}");
    }
    let host = &value["HostConfig"];
    let id = value["Id"].as_str().context("missing container identity")?;
    let static_dir = value["StaticDir"]
        .as_str()
        .context("missing private container storage identity")?;
    if id.len() != 64
        || !id
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        || !static_dir.starts_with('/')
        || static_dir.split('/').any(|v| matches!(v, "." | ".."))
        || !static_dir.ends_with(&format!("/overlay-containers/{id}/userdata"))
        || host["LogConfig"]
            != json!({"Type":"k8s-file","Config":null,"Path":format!("{static_dir}/ctr.log"),"Tag":"","Size":"4MB"})
    {
        bail!("Penpot container bounded logging configuration has drifted: {alias}");
    }
    let empty = |v: &Value| v.is_null() || v.as_array().is_some_and(Vec::is_empty);
    let dropped: std::collections::BTreeSet<_> = host["CapDrop"]
        .as_array()
        .context("missing dropped capabilities")?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let expected_dropped = [
        "CAP_CHOWN",
        "CAP_DAC_OVERRIDE",
        "CAP_FOWNER",
        "CAP_FSETID",
        "CAP_KILL",
        "CAP_NET_BIND_SERVICE",
        "CAP_SETFCAP",
        "CAP_SETGID",
        "CAP_SETPCAP",
        "CAP_SETUID",
        "CAP_SYS_CHROOT",
    ]
    .into_iter()
    .collect();
    let expected_tmp: BTreeMap<_, _> = tmpfs(alias)
        .into_iter()
        .map(|(k, v)| (k, format!("{v},rprivate,tmpcopyup")))
        .collect();
    if host["Privileged"] != false
        || host["ReadonlyRootfs"] != true
        || host["NetworkMode"] != "bridge"
        || host["PublishAllPorts"] != false
        || host["RestartPolicy"]["Name"] != "no"
        || !empty(&host["CapAdd"])
        || dropped != expected_dropped
        || !empty(&value["EffectiveCaps"])
        || !empty(&value["BoundingCaps"])
        || !empty(&host["Devices"])
        || !empty(&host["DeviceRequests"])
        || host["SecurityOpt"] != json!(["no-new-privileges"])
        || host["PidMode"] != "private"
        || host["IpcMode"] != "private"
        || host["UTSMode"] != "private"
        || !config_string(&host["UsernsMode"])?.is_empty()
        || !empty(&host["ExtraHosts"])
        || !empty(&host["VolumesFrom"])
        || host["Memory"] != p.memory_mib * 1024 * 1024
        || host["MemorySwap"] != p.memory_mib * 1024 * 1024
        || host["NanoCpus"] != p.cpus * 1_000_000_000
        || host["PidsLimit"] != p.pids
        || host["Tmpfs"] != serde_json::to_value(expected_tmp)?
    {
        bail!("Penpot container isolation/resource configuration has drifted: {alias}");
    }
    let ports = &host["PortBindings"];
    if alias == "penpot-frontend" {
        if *ports != json!({"8080/tcp":[{"HostIp":"127.0.0.1","HostPort":state.port.to_string()}]})
        {
            bail!("Penpot frontend must expose only its selected loopback port");
        }
        if value["State"]["Running"] == true && value["NetworkSettings"]["Ports"] != *ports {
            bail!("Penpot frontend loopback port is configured but not published by Podman");
        }
    } else if !ports.is_null() && ports.as_object().is_none_or(|v| !v.is_empty()) {
        bail!("internal Penpot service unexpectedly exposes a host port: {alias}");
    }
    let networks = value["NetworkSettings"]["Networks"]
        .as_object()
        .context("missing service network")?;
    if networks.len() != 1
        || !networks.contains_key(&state.owner)
        || networks[&state.owner]["Aliases"]
            .as_array()
            .is_none_or(|v| !v.iter().any(|v| v.as_str() == Some(alias)))
    {
        bail!("Penpot container network/alias has drifted: {alias}");
    }
    let expected_mount = match alias {
        "penpot-postgres" => Some(("db", "/var/lib/postgresql/data", true)),
        "penpot-valkey" => Some(("cache", "/data", true)),
        "penpot-backend" => Some(("assets", "/opt/data/assets", true)),
        "penpot-frontend" => Some(("assets", "/opt/data/assets", false)),
        _ => None,
    };
    let mounts = value["Mounts"]
        .as_array()
        .context("missing container mount metadata")?;
    if mounts.len() != usize::from(expected_mount.is_some()) {
        bail!("Penpot container has unexpected mounts: {alias}");
    }
    if let Some((suffix, target, rw)) = expected_mount {
        let mount = &mounts[0];
        if mount["Type"] != "volume"
            || mount["Name"] != format!("{}-{suffix}", state.owner)
            || mount["Destination"] != target
            || mount["RW"] != rw
        {
            bail!("Penpot container volume has drifted: {alias}");
        }
    }
    let expected_binds = if let Some((suffix, target, rw)) = expected_mount {
        json!([format!(
            "{}-{suffix}:{target}:{}",
            state.owner,
            if rw {
                "rw,U,rprivate,nosuid,nodev,rbind"
            } else {
                "ro,rprivate,nosuid,nodev,rbind"
            }
        )])
    } else {
        json!([])
    };
    if host["Binds"] != expected_binds {
        bail!("Penpot mount source/options have drifted: {alias}");
    }
    Ok(())
}
fn check_container(binary: &Engine, value: &Value, state: &State, alias: &str) -> Result<()> {
    let image = inspected(binary, "image", &qualified_image(&state.images[alias]))?;
    validate_container(value, &image, state, alias)
}
fn validate_resource(kind: &str, value: &Value) -> Result<()> {
    if kind == "network" {
        if value["internal"] != true
            || value["driver"] != "bridge"
            || value["ipv6_enabled"] != false
            || value["dns_enabled"] != true
            || value["ipam_options"] != json!({"driver":"host-local"})
            || value["options"].as_object().is_some_and(|v| !v.is_empty())
        {
            bail!("owned Penpot network configuration has drifted");
        }
    } else if value["Driver"] != "local"
        || (!value["Options"].is_null()
            && value["Options"].as_object().is_none_or(|v| !v.is_empty()))
    {
        bail!("owned Penpot volume driver/options have drifted");
    }
    Ok(())
}
fn check_resources(binary: &Engine, state: &State, require_present: bool) -> Result<()> {
    for (kind, name) in [
        ("network", state.owner.clone()),
        ("volume", format!("{}-db", state.owner)),
        ("volume", format!("{}-cache", state.owner)),
        ("volume", format!("{}-assets", state.owner)),
    ] {
        if let Some(value) = own_resource(binary, kind, &name, &state.owner)? {
            validate_resource(kind, &value)?;
        } else if require_present {
            bail!("running Penpot service is missing its owned {kind}: {name}");
        }
    }
    Ok(())
}
fn ensure_resources(binary: &Engine, state: &State) -> Result<()> {
    if let Some(network) = own_resource(binary, "network", &state.owner, &state.owner)? {
        validate_resource("network", &network)?;
    } else {
        run(
            binary,
            [
                "network".into(),
                "create".into(),
                "--driver=bridge".into(),
                "--internal".into(),
                "--label".into(),
                format!("{LABEL}={}", state.owner),
                state.owner.clone(),
            ],
        )?;
    }
    for suffix in ["db", "cache", "assets"] {
        let name = format!("{}-{suffix}", state.owner);
        if let Some(volume) = own_resource(binary, "volume", &name, &state.owner)? {
            validate_resource("volume", &volume)?;
        } else {
            run(
                binary,
                [
                    "volume".into(),
                    "create".into(),
                    "--label".into(),
                    format!("{LABEL}={}", state.owner),
                    name,
                ],
            )?;
        }
    }
    Ok(())
}
fn launch_container(paths: &Paths, binary: &Engine, state: &State, alias: &str) -> Result<()> {
    let name = container(&state.owner, alias);
    if let Some(value) = own_resource(binary, "container", &name, &state.owner)? {
        check_container(binary, &value, state, alias)?;
        if value["State"]["Running"] != true {
            run(binary, ["container".into(), "start".into(), name])?;
        }
        return Ok(());
    }
    let mut envfile = tempfile::NamedTempFile::new_in(data(paths))?;
    for (key, val) in service_env(state, alias) {
        writeln!(envfile, "{key}={val}")?;
    }
    let p = policy(alias);
    let mut args = vec![
        "container".into(),
        "run".into(),
        "--detach".into(),
        "--pull=never".into(),
        "--platform=linux/arm64".into(),
        "--name".into(),
        name,
        "--hostname".into(),
        alias.into(),
        "--network".into(),
        state.owner.clone(),
        "--network-alias".into(),
        alias.into(),
        "--label".into(),
        format!("{LABEL}={}", state.owner),
        "--restart=no".into(),
        "--log-driver=k8s-file".into(),
        "--log-opt=max-size=4m".into(),
        "--env-file".into(),
        envfile.path().to_string_lossy().into_owned(),
        format!("--user={}:{}", p.uid, p.uid),
        "--read-only".into(),
        "--read-only-tmpfs=false".into(),
        "--cap-drop=ALL".into(),
        "--security-opt=no-new-privileges".into(),
        "--ipc=private".into(),
        format!("--memory={}m", p.memory_mib),
        format!("--memory-swap={}m", p.memory_mib),
        format!("--cpus={}", p.cpus),
        format!("--pids-limit={}", p.pids),
        "--http-proxy=false".into(),
    ];
    for (path, options) in tmpfs(alias) {
        args.extend(["--tmpfs".into(), format!("{path}:{options}")]);
    }
    if alias == "penpot-frontend" {
        args.extend([
            "--publish".into(),
            format!("127.0.0.1:{}:8080", state.port),
            "--entrypoint=/bin/bash".into(),
        ]);
    }
    let mount = match alias {
        "penpot-postgres" => Some(("db", "/var/lib/postgresql/data", true)),
        "penpot-valkey" => Some(("cache", "/data", true)),
        "penpot-backend" => Some(("assets", "/opt/data/assets", true)),
        "penpot-frontend" => Some(("assets", "/opt/data/assets", false)),
        _ => None,
    };
    if let Some((suffix, target, rw)) = mount {
        args.extend([
            "--volume".into(),
            format!(
                "{}-{suffix}:{target}:{}",
                state.owner,
                if rw { "rw,U" } else { "ro" }
            ),
        ]);
    }
    args.push(qualified_image(&state.images[alias]));
    if matches!(
        alias,
        "penpot-postgres" | "penpot-valkey" | "penpot-frontend"
    ) {
        let image = inspected(binary, "image", &qualified_image(&state.images[alias]))?;
        args.extend(
            expected_command(&image, alias)
                .as_array()
                .context("fixed service command")?
                .iter()
                .map(|v| v.as_str().unwrap().to_owned()),
        );
    }
    run(binary, args)?;
    Ok(())
}
fn remove_containers(binary: &Engine, state: &State) -> Result<()> {
    for alias in ALIASES.into_iter().rev() {
        let name = container(&state.owner, alias);
        if own_resource(binary, "container", &name, &state.owner)?.is_some() {
            run(
                binary,
                ["container".into(), "rm".into(), "--force".into(), name],
            )?;
        }
    }
    Ok(())
}

fn backend_ready(mut response: reqwest::blocking::Response) -> bool {
    if !response.status().is_success()
        || !response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| v.starts_with("application/json"))
    {
        return false;
    }
    let mut bytes = vec![];
    if response
        .by_ref()
        .take(65537)
        .read_to_end(&mut bytes)
        .is_err()
        || bytes.len() > 65536
    {
        return false;
    }
    serde_json::from_slice::<Value>(&bytes).is_ok_and(|v| {
        v["id"] == "00000000-0000-0000-0000-000000000000" && v["fullname"] == "Anonymous User"
    })
}

pub(super) fn start(
    paths: &Paths,
    registry: &Registry,
    id: &str,
    port: Option<u16>,
) -> Result<Value> {
    let package = registry.get(id)?;
    let service = package
        .service
        .as_ref()
        .context("package has no local services")?;
    super::store::verify(paths, registry, id)?;
    verify(paths, package)?;
    let bin = executable(paths)?;
    util::ensure_directory(&data(paths))?;
    let _lock = util::FileLock::acquire(&data(paths).join("operation.lock"))?;
    let mut saved = if data(paths).join("state.json").exists() {
        bound_state(paths, &bin)?
    } else {
        let port = if let Some(port) = port {
            if port <= 1024 {
                bail!("service port must exceed 1024");
            }
            port
        } else {
            let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))?;
            listener.local_addr()?.port()
        };
        State {
            schema_version: 1,
            owner: owner(paths),
            engine_owner: Some(bin.owner()),
            port,
            database_password: secret()?,
            service_secret: format!("{}{}", secret()?, secret()?),
            images: service.images.clone(),
        }
    };
    if port.is_some_and(|p| p != saved.port) || saved.images != service.images {
        for alias in ALIASES {
            if own_resource(
                &bin,
                "container",
                &container(&saved.owner, alias),
                &saved.owner,
            )?
            .is_some()
            {
                bail!("stop Penpot before changing service port or image generation");
            }
        }
        if let Some(port) = port {
            if port <= 1024 {
                bail!("service port must exceed 1024");
            }
            saved.port = port;
        }
        saved.images = service.images.clone();
    }
    let mut previous = BTreeMap::new();
    for alias in ALIASES {
        let value = own_resource(
            &bin,
            "container",
            &container(&saved.owner, alias),
            &saved.owner,
        )?;
        if let Some(value) = &value {
            check_container(&bin, value, &saved, alias)?;
        }
        previous.insert(alias, value.map(|v| v["State"]["Running"] == true));
    }
    // Preflight the full ownership set before the state or any resource changes.
    check_resources(&bin, &saved, false)?;
    save_state(paths, &saved)?;
    let outcome = (|| {
        ensure_resources(&bin, &saved)?;
        launch_container(paths, &bin, &saved, "penpot-postgres")?;
        launch_container(paths, &bin, &saved, "penpot-valkey")?;
        let deadline = Instant::now() + Duration::from_secs(45);
        loop {
            let pg = run(
                &bin,
                vec![
                    "exec".into(),
                    container(&saved.owner, "penpot-postgres"),
                    "pg_isready".into(),
                    "-U".into(),
                    "penpot".into(),
                ],
            );
            let kv = run(
                &bin,
                vec![
                    "exec".into(),
                    container(&saved.owner, "penpot-valkey"),
                    "valkey-cli".into(),
                    "ping".into(),
                ],
            );
            if pg.is_ok() && kv.as_ref().is_ok_and(|s| s.trim() == "PONG") {
                break;
            }
            if Instant::now() >= deadline {
                bail!("Penpot database services did not become ready");
            }
            std::thread::sleep(Duration::from_millis(250));
        }
        for alias in &ALIASES[2..] {
            launch_container(paths, &bin, &saved, alias)?;
        }
        // Nginx resolves the fixed upstream hostname at process start. When a
        // missing backend is recreated, refresh only our exact owned frontend.
        if previous["penpot-backend"].is_none() && previous["penpot-frontend"] == Some(true) {
            run(
                &bin,
                [
                    "container".into(),
                    "restart".into(),
                    container(&saved.owner, "penpot-frontend"),
                ],
            )?;
        }
        let client = reqwest::blocking::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(3))
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        let deadline = Instant::now() + Duration::from_secs(120);
        loop {
            let ready = client
                .get(format!("{}/api/main/methods/get-profile", url(&saved)))
                .header("Accept", "application/json")
                .send()
                .is_ok_and(backend_ready);
            if ready {
                break;
            }
            if Instant::now() >= deadline {
                bail!(
                    "Penpot frontend/backend did not become ready; check Podman resources and retry mirket package start penpot"
                );
            }
            std::thread::sleep(Duration::from_millis(250));
        }
        for alias in ALIASES {
            let value = own_resource(
                &bin,
                "container",
                &container(&saved.owner, alias),
                &saved.owner,
            )?
            .context("Penpot service disappeared during startup")?;
            check_container(&bin, &value, &saved, alias)?;
            if value["State"]["Running"] != true {
                bail!("Penpot service exited during startup: {alias}");
            }
        }
        Ok::<_, anyhow::Error>(())
    })();
    if let Err(error) = outcome {
        // Keep only bounded state diagnostics: startup cleanup removes new containers,
        // so a later logs command cannot explain them. Do not emit private environment.
        let mut failed_states = vec![];
        for alias in ALIASES {
            if let Ok(Some(value)) = own_resource(
                &bin,
                "container",
                &container(&saved.owner, alias),
                &saved.owner,
            ) {
                failed_states.push(json!({"service":alias,"running":value["State"]["Running"],"exit_code":value["State"]["ExitCode"]}));
            }
        }
        let error = error.context(format!("startup service states: {}", json!(failed_states)));
        let recovery = (|| {
            for alias in ALIASES.into_iter().rev() {
                let name = container(&saved.owner, alias);
                if own_resource(&bin, "container", &name, &saved.owner)?.is_some() {
                    match previous[alias] {
                        None => {
                            run(
                                &bin,
                                ["container".into(), "rm".into(), "--force".into(), name],
                            )?;
                        }
                        Some(false) => {
                            run(
                                &bin,
                                [
                                    "container".into(),
                                    "stop".into(),
                                    "--time".into(),
                                    "5".into(),
                                    name,
                                ],
                            )?;
                        }
                        Some(true) => {}
                    }
                }
            }
            Ok::<_, anyhow::Error>(())
        })();
        return Err(error).context(match recovery {Ok(())=>"Penpot start failed; new containers removed, prior running state and durable data retained".into(),Err(e)=>format!("Penpot start failed; cleanup also failed: {e:#}; reconcile with mirket package stop penpot")});
    }
    Ok(
        json!({"package":id,"running":true,"url":url(&saved),"headless":"Browser-backed design operations require an active pinned Chromium session; no manual window is needed.","data":"Durable Podman volumes and private service credentials are retained under the selected Mirket home."}),
    )
}
pub(super) fn stop(paths: &Paths, id: &str, purge_data: bool) -> Result<Value> {
    if id != "penpot" {
        bail!("package has no local services");
    }
    let bin = executable(paths)?;
    util::ensure_directory(&data(paths))?;
    let _lock = util::FileLock::acquire(&data(paths).join("operation.lock"))?;
    let saved = bound_state(paths, &bin)?;
    // Validate the whole ownership set before any deletion. Explicit purge is limited
    // to this home's three labeled volumes and private service directory.
    for alias in ALIASES {
        own_resource(
            &bin,
            "container",
            &container(&saved.owner, alias),
            &saved.owner,
        )?;
    }
    own_resource(&bin, "network", &saved.owner, &saved.owner)?;
    for suffix in ["db", "cache", "assets"] {
        own_resource(
            &bin,
            "volume",
            &format!("{}-{suffix}", saved.owner),
            &saved.owner,
        )?;
    }
    remove_containers(&bin, &saved)?;
    if own_resource(&bin, "network", &saved.owner, &saved.owner)?.is_some() {
        run(&bin, ["network".into(), "rm".into(), saved.owner.clone()])?;
    }
    if purge_data {
        for suffix in ["db", "cache", "assets"] {
            let name = format!("{}-{suffix}", saved.owner);
            if own_resource(&bin, "volume", &name, &saved.owner)?.is_some() {
                run(&bin, ["volume".into(), "rm".into(), name])?;
            }
        }
        let browser = data(paths).join("browser");
        util::reject_symlink_components(&browser)?;
        if browser.exists() {
            fs::remove_dir_all(browser)?;
        }
        fs::remove_file(data(paths).join("state.json"))?;
    }
    Ok(json!({"package":id,"running":false,"durable_data_retained":!purge_data}))
}
pub(super) fn status(paths: &Paths, package: &Package) -> Result<Value> {
    verify(paths, package)?;
    let bin = executable(paths)?;
    if !data(paths).join("state.json").exists() {
        return Ok(json!({"running":false,"action":"mirket package start penpot"}));
    }
    let saved = bound_state(paths, &bin)?;
    let mut running = true;
    let mut containers = vec![];
    for alias in ALIASES {
        let value = own_resource(
            &bin,
            "container",
            &container(&saved.owner, alias),
            &saved.owner,
        )?;
        if let Some(value) = &value {
            check_container(&bin, value, &saved, alias)?;
        }
        let ready = value
            .as_ref()
            .is_some_and(|v| v["State"]["Running"] == true);
        running &= ready;
        containers.push(json!({"service":alias,"running":ready}));
    }
    check_resources(&bin, &saved, running)?;
    if package
        .service
        .as_ref()
        .is_some_and(|s| s.images != saved.images)
    {
        running = false;
    }
    Ok(
        json!({"running":running,"url":url(&saved),"containers":containers,"action":if running {"mirket package probe penpot"}else{"mirket package start penpot"}}),
    )
}
pub(super) fn launch_env(paths: &Paths, package: &Package) -> Result<BTreeMap<String, String>> {
    let health = status(paths, package)?;
    if health["running"] != true {
        bail!("Penpot local services are not running; use mirket package start penpot");
    }
    let saved = state(paths)?;
    let browser_data = data(paths).join("browser");
    util::ensure_directory(&browser_data)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&browser_data, fs::Permissions::from_mode(0o700))?;
    }
    Ok(BTreeMap::from([
        ("MIRKET_PENPOT_URL".into(), url(&saved)),
        (
            "MIRKET_PENPOT_DATA".into(),
            browser_data.to_string_lossy().into_owned(),
        ),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mutable_tags_and_incomplete_profiles_are_refused() {
        let mut s = Service {
            profile: "penpot".into(),
            images: BTreeMap::new(),
        };
        assert!(validate(&s).is_err());
        for a in ALIASES {
            s.images.insert(a.into(), "repo:latest".into());
        }
        assert!(validate(&s).is_err());
    }
    #[test]
    fn listener_and_success_exit_are_not_backend_readiness() {
        for (content_type, body, expected) in [
            ("text/html", "<h1>Penpot</h1>", false),
            ("application/json", "{}", false),
            (
                "application/json",
                "{\"id\":\"wrong\",\"fullname\":\"Anonymous User\"}",
                false,
            ),
            (
                "application/json",
                "{\"id\":\"00000000-0000-0000-0000-000000000000\",\"fullname\":\"Anonymous User\"}",
                true,
            ),
        ] {
            let socket = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
            let port = socket.local_addr().unwrap().port();
            let server = std::thread::spawn(move || {
                let (mut stream, _) = socket.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut request = [0; 2048];
                let _ = stream.read(&mut request);
                write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
            });
            let response = reqwest::blocking::Client::builder()
                .no_proxy()
                .timeout(Duration::from_secs(3))
                .build()
                .unwrap()
                .get(format!("http://127.0.0.1:{port}"))
                .send()
                .unwrap();
            assert_eq!(backend_ready(response), expected);
            server.join().unwrap();
        }
    }
    #[cfg(unix)]
    #[test]
    fn state_is_bound_to_its_disposable_home() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        let pa = Paths::new(a.path().into()).unwrap();
        let pb = Paths::new(b.path().into()).unwrap();
        let s = State {
            schema_version: 1,
            owner: owner(&pa),
            engine_owner: Some("test-engine".into()),
            port: 3000,
            database_password: secret().unwrap(),
            service_secret: format!("{}{}", secret().unwrap(), secret().unwrap()),
            images: Registry::embedded()
                .unwrap()
                .get("penpot")
                .unwrap()
                .service
                .as_ref()
                .unwrap()
                .images
                .clone(),
        };
        save_state(&pa, &s).unwrap();
        assert!(state(&pa).is_ok());
        let wrong_engine = Engine {
            binary: PathBuf::from("/unused"),
            env: BTreeMap::new(),
            staging: pa.root.join("execution/podman/staging"),
            connection: "other-environment".into(),
        };
        assert!(bound_state(&pa, &wrong_engine).is_err());
        assert_eq!(
            state(&pa).unwrap().engine_owner.as_deref(),
            Some("test-engine")
        );
        util::ensure_directory(&data(&pb)).unwrap();
        fs::copy(data(&pa).join("state.json"), data(&pb).join("state.json")).unwrap();
        assert!(state(&pb).is_err());
        let mut bad = s;
        bad.images.clear();
        save_state(&pa, &bad).unwrap();
        assert!(
            state(&pa).is_err(),
            "incomplete state must not reach indexing"
        );
        bad.images = Registry::embedded()
            .unwrap()
            .get("penpot")
            .unwrap()
            .service
            .as_ref()
            .unwrap()
            .images
            .clone();
        bad.database_password.replace_range(0..1, "\n");
        save_state(&pa, &bad).unwrap();
        assert!(state(&pa).is_err(), "state cannot inject env-file lines");
    }

    #[test]
    fn unbound_service_state_is_preserved_and_refused() {
        let root = tempfile::tempdir().unwrap();
        let paths = Paths::new(root.path().into()).unwrap();
        let saved = State {
            schema_version: 1,
            owner: owner(&paths),
            engine_owner: None,
            port: 3010,
            database_password: "a".repeat(64),
            service_secret: "b".repeat(128),
            images: Registry::embedded()
                .unwrap()
                .get("penpot")
                .unwrap()
                .service
                .as_ref()
                .unwrap()
                .images
                .clone(),
        };
        save_state(&paths, &saved).unwrap();
        let before = fs::read(data(&paths).join("state.json")).unwrap();
        let document = data(&paths).join("user-design.penpot");
        fs::write(&document, b"private design").unwrap();
        assert!(state(&paths).unwrap_err().to_string().contains("not bound"));
        assert_eq!(fs::read(data(&paths).join("state.json")).unwrap(), before);
        assert_eq!(fs::read(document).unwrap(), b"private design");
    }
    #[test]
    fn podman_policy_and_identity_reject_plausible_drift() {
        let paths = Paths::new(std::env::temp_dir()).unwrap();
        let state = State {
            schema_version: 1,
            owner: owner(&paths),
            engine_owner: Some("test-engine".into()),
            port: 3010,
            database_password: "a".repeat(64),
            service_secret: "b".repeat(128),
            images: Registry::embedded()
                .unwrap()
                .get("penpot")
                .unwrap()
                .service
                .as_ref()
                .unwrap()
                .images
                .clone(),
        };
        let image = json!({"Id":"sha256:local-image","Config":{"Cmd":["start"],"Entrypoint":null,"Env":["PATH=/bin"]}});
        for alias in ALIASES {
            let p = policy(alias);
            let mut env = vec![json!("PATH=/bin")];
            env.extend(
                service_env(&state, alias)
                    .into_iter()
                    .map(|(k, v)| json!(format!("{k}={v}"))),
            );
            let tmp: BTreeMap<_, _> = tmpfs(alias)
                .into_iter()
                .map(|(k, v)| (k, format!("{v},rprivate,tmpcopyup")))
                .collect();
            let ports = if alias == "penpot-frontend" {
                json!({"8080/tcp":[{"HostIp":"127.0.0.1","HostPort":"3010"}]})
            } else {
                Value::Null
            };
            let mounts = match alias {
                "penpot-postgres" => {
                    json!([{"Type":"volume","Name":format!("{}-db",state.owner),"Destination":"/var/lib/postgresql/data","RW":true}])
                }
                "penpot-valkey" => {
                    json!([{"Type":"volume","Name":format!("{}-cache",state.owner),"Destination":"/data","RW":true}])
                }
                "penpot-backend" | "penpot-frontend" => {
                    json!([{"Type":"volume","Name":format!("{}-assets",state.owner),"Destination":"/opt/data/assets","RW":alias=="penpot-backend"}])
                }
                _ => json!([]),
            };
            let mut good = json!({"Image":image["Id"],"Config":{"Image":qualified_image(&state.images[alias]),"Cmd":expected_command(&image,alias),"Entrypoint":expected_entrypoint(&image,alias),"User":format!("{}:{}",p.uid,p.uid),"Hostname":alias,"WorkingDir":"/","StopSignal":"SIGTERM","Env":env},"HostConfig":{"Privileged":false,"ReadonlyRootfs":true,"NetworkMode":"bridge","PublishAllPorts":false,"RestartPolicy":{"Name":"no"},"CapDrop":["CAP_CHOWN","CAP_DAC_OVERRIDE","CAP_FOWNER","CAP_FSETID","CAP_KILL","CAP_NET_BIND_SERVICE","CAP_SETFCAP","CAP_SETGID","CAP_SETPCAP","CAP_SETUID","CAP_SYS_CHROOT"],"SecurityOpt":["no-new-privileges"],"PidMode":"private","IpcMode":"private","UTSMode":"private","UsernsMode":"","Memory":p.memory_mib*1024*1024,"MemorySwap":p.memory_mib*1024*1024,"NanoCpus":p.cpus*1_000_000_000,"PidsLimit":p.pids,"Tmpfs":tmp,"PortBindings":ports},"State":{"Running":true},"NetworkSettings":{"Ports":ports,"Networks":{state.owner.clone():{"Aliases":[alias]}}},"Mounts":mounts});
            good["HostConfig"]["Binds"] =
                if let Some(m) = good["Mounts"].as_array().unwrap().first() {
                    json!([format!(
                        "{}:{}:{}",
                        m["Name"].as_str().unwrap(),
                        m["Destination"].as_str().unwrap(),
                        if m["RW"] == true {
                            "rw,U,rprivate,nosuid,nodev,rbind"
                        } else {
                            "ro,rprivate,nosuid,nodev,rbind"
                        }
                    )])
                } else {
                    json!([])
                };
            good["Id"] = "c".repeat(64).into();
            good["StaticDir"] = format!(
                "/var/home/core/.local/share/containers/storage/overlay-containers/{}/userdata",
                "c".repeat(64)
            )
            .into();
            good["HostConfig"]["LogConfig"] = json!({"Type":"k8s-file","Config":null,"Path":format!("{}/ctr.log",good["StaticDir"].as_str().unwrap()),"Tag":"","Size":"4MB"});
            validate_container(&good, &image, &state, alias).unwrap();
            for (field, bad) in [
                ("Privileged", json!(true)),
                ("ReadonlyRootfs", json!(false)),
                ("SecurityOpt", json!([])),
                ("CapAdd", json!(["CAP_SYS_ADMIN"])),
                ("Memory", json!(0)),
                ("MemorySwap", json!(0)),
                ("PidsLimit", json!(0)),
                ("PidMode", json!("host")),
                ("IpcMode", json!("host")),
                ("Binds", json!(["/Users:/host"])),
                ("LogConfig", json!({"Type":"journald","Size":"4MB"})),
            ] {
                let mut drift = good.clone();
                drift["HostConfig"][field] = bad;
                assert!(
                    validate_container(&drift, &image, &state, alias).is_err(),
                    "accepted {alias} drift {field}"
                );
            }
            for (field, bad) in [
                ("Size", json!("0B")),
                ("Path", json!("/tmp/unrelated.log")),
                ("Tag", json!("unreviewed")),
                ("Config", json!({"path":"/tmp/unrelated.log"})),
            ] {
                let mut drift = good.clone();
                drift["HostConfig"]["LogConfig"][field] = bad;
                assert!(
                    validate_container(&drift, &image, &state, alias).is_err(),
                    "accepted {alias} logging drift {field}"
                );
            }
            let mut bad = good.clone();
            bad["Config"]["User"] = "0:0".into();
            assert!(validate_container(&bad, &image, &state, alias).is_err());
            let mut bad = good.clone();
            bad["Config"]["Cmd"] = json!(["unreviewed"]);
            assert!(validate_container(&bad, &image, &state, alias).is_err());
            let mut bad = good.clone();
            bad["NetworkSettings"]["Networks"]["unrelated"] = json!({});
            assert!(validate_container(&bad, &image, &state, alias).is_err());
            if alias == "penpot-frontend" {
                let mut bad = good.clone();
                bad["Mounts"][0]["RW"] = true.into();
                assert!(validate_container(&bad, &image, &state, alias).is_err());
                let mut bad = good.clone();
                bad["NetworkSettings"]["Ports"] = json!({"8080/tcp":[]});
                assert!(validate_container(&bad, &image, &state, alias).is_err());
                let env = service_env(&state, alias);
                assert!(!env.contains_key("PENPOT_DATABASE_PASSWORD"));
                assert!(!env.contains_key("PENPOT_SECRET_KEY"));
            }
        }
    }
    #[test]
    fn only_internal_podman_networks_and_local_owned_volumes_are_valid() {
        let good = json!({"internal":true,"driver":"bridge","ipv6_enabled":false,"dns_enabled":true,"ipam_options":{"driver":"host-local"}});
        validate_resource("network", &good).unwrap();
        for (k, v) in [
            ("internal", json!(false)),
            ("driver", json!("macvlan")),
            ("ipv6_enabled", json!(true)),
            ("options", json!({"unreviewed":"true"})),
        ] {
            let mut bad = good.clone();
            bad[k] = v;
            assert!(validate_resource("network", &bad).is_err());
        }
        validate_resource("volume", &json!({"Driver":"local","Options":{}})).unwrap();
        assert!(
            validate_resource(
                "volume",
                &json!({"Driver":"local","Options":{"device":"remote:/private"}})
            )
            .is_err()
        );
    }
}
