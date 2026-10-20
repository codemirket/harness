//! Disposable native backend compatibility check. Never opens saved service data.
use super::*;

const LABEL: &str = "org.mirket.backend-check";

fn profile(value: &str) -> Result<Value> {
    let value: Value = serde_json::from_str(value)?;
    ensure!(
        value.is_object() && value["id"].is_string(),
        "invalid profile response"
    );
    Ok(value)
}

fn wait_profile(engine: &Engine, container: &str) -> Result<Value> {
    let deadline = Instant::now() + Duration::from_secs(180);
    loop {
        let result = engine
            .run(
                &[
                    "exec",
                    container,
                    "curl",
                    "-fsS",
                    "--max-time",
                    "3",
                    "-H",
                    "Accept: application/json",
                    "http://127.0.0.1:6060/api/main/methods/get-profile",
                ]
                .map(str::to_owned),
                Duration::from_secs(5),
            )
            .and_then(|v| profile(&v));
        if result.is_ok() || Instant::now() >= deadline {
            return result.context("backend profile readiness");
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

pub(crate) fn check(paths: &Paths, output: &Path) -> Result<Value> {
    util::reject_symlink_components(output)?;
    ensure!(
        output.is_absolute() && !output.exists(),
        "new absolute evidence directory required"
    );
    fs::create_dir(output)?;
    let engine = Engine::resolve(paths)?;
    engine.ready()?;
    let registry = definition()?;
    let backend = candidate_image(paths, "penpot-backend")?;
    let stage = tempfile::tempdir()?;
    let stage_path = stage.path().canonicalize()?;
    let owner = format!("mirket-check-{}", secret(12)?);
    let network = format!("{owner}-network");
    let mut state = State {
        schema_version: 1,
        owner: owner.clone(),
        engine_owner: engine.owner(),
        registry_sha256: registry_hash(),
        images: BTreeMap::new(),
        published_ports: BTreeMap::new(),
        groups: BTreeSet::new(),
        database_password: secret(32)?,
        service_secret: secret(32)?,
        penpot_email: "fixture@mirket.invalid".into(),
        penpot_password: secret(32)?,
    };
    state.images.insert("penpot-backend".into(), backend);
    for alias in ["penpot-postgres", "penpot-valkey"] {
        let reference = registry["images"][alias].as_str().context("service pin")?;
        // A fresh check may pull only the reviewed immutable fixture dependencies.
        engine.run(&["pull".into(), reference.into()], Duration::from_secs(300))?;
        state
            .images
            .insert(alias.into(), image_id(&engine, reference)?);
    }
    let mut created = Vec::new();
    let mut network_created = false;
    let outcome = (|| -> Result<Value> {
        engine.run(
            &[
                "network".into(),
                "create".into(),
                "--internal".into(),
                "--label".into(),
                format!("{LABEL}={owner}"),
                network.clone(),
            ],
            Duration::from_secs(30),
        )?;
        network_created = true;
        for alias in ["penpot-postgres", "penpot-valkey", "penpot-backend"] {
            let name = format!("{owner}-{alias}");
            let user = uid(alias);
            let (memory, cpus, pids) = limits(alias);
            let env_path = stage_path.join(alias);
            let mut environment = env_for(&state, alias);
            if alias == "penpot-postgres" {
                // Unprivileged init owns the subdirectory it creates. Podman
                // does not accept uid/gid in --tmpfs options.
                environment.insert("PGDATA".into(), "/var/lib/postgresql/data/db".into());
            }
            save_private(
                &env_path,
                environment
                    .iter()
                    .map(|(k, v)| format!("{k}={v}\n"))
                    .collect::<String>()
                    .as_bytes(),
            )?;
            let data = match alias {
                "penpot-postgres" => "/var/lib/postgresql/data",
                "penpot-valkey" => "/data",
                _ => "/opt/data/assets",
            };
            let mut args = vec![
                "run".into(),
                "-d".into(),
                "--pull=never".into(),
                "--name".into(),
                name.clone(),
                "--network".into(),
                network.clone(),
                "--network-alias".into(),
                alias.into(),
                "--label".into(),
                format!("{LABEL}={owner}"),
                format!("--userns=keep-id:uid={user},gid={user}"),
                format!("--user={user}:{user}"),
                "--read-only".into(),
                "--cap-drop=ALL".into(),
                "--security-opt=no-new-privileges".into(),
                format!("--memory={memory}"),
                format!("--memory-swap={memory}"),
                format!("--cpus={cpus}"),
                format!("--pids-limit={pids}"),
                "--restart=no".into(),
                "--http-proxy=false".into(),
                "--log-driver=k8s-file".into(),
                "--log-opt=max-size=4m".into(),
                "--tmpfs=/tmp:rw,nosuid,nodev,size=256m,mode=1777".into(),
                "--tmpfs=/run:rw,nosuid,nodev,size=64m,mode=1777".into(),
                format!("--tmpfs={data}:rw,nosuid,nodev,size=256m,mode=1777"),
                "--env-file".into(),
                env_path.to_string_lossy().into_owned(),
                state.images[alias].clone(),
            ];
            if let Some(command) = service_command(alias) {
                args.extend(command.into_iter().map(str::to_owned));
            }
            // Register intended names before dispatch so an uncertain create is inspected at cleanup.
            created.push(name);
            engine.run(&args, Duration::from_secs(60))?;
        }
        let name = format!("{owner}-penpot-backend");
        let first = wait_profile(&engine, &name)?;
        let runtime = engine.run(&["exec", &name, "/bin/bash", "-ec",
            "test -s /var/lib/dpkg/status; java -version 2>&1; magick -version; dpkg-query -W | wc -l"]
            .map(str::to_owned), Duration::from_secs(30))?;
        let postgres = format!("{owner}-penpot-postgres");
        let tables = engine.run(&["exec", &postgres, "psql", "-U", "penpot", "-d", "penpot", "-At", "-c",
            "SELECT count(*) FROM information_schema.tables WHERE table_schema='public' AND table_type='BASE TABLE'"]
            .map(str::to_owned), Duration::from_secs(20))?;
        ensure!(
            tables.trim().parse::<u64>()? > 20,
            "database migrations absent"
        );
        engine.run(&["restart".into(), name.clone()], Duration::from_secs(60))?;
        let second = wait_profile(&engine, &name)?;
        ensure!(
            first["id"] == second["id"],
            "anonymous profile changed after restart"
        );
        Ok(
            json!({"profile":first,"restart_profile":second,"public_tables":tables.trim(),
            "native_runtime":runtime}),
        )
    })();
    let mut cleanup_errors = Vec::new();
    for name in created.iter().rev() {
        if let Ok(value) = inspect(&engine, "container", name) {
            if value["Config"]["Labels"][LABEL] != owner {
                cleanup_errors.push(format!("ownership mismatch: {name}"));
                continue;
            }
            if let Ok(logs) = engine.container_logs(name)
                && let Err(error) = fs::write(output.join(format!("{name}.log")), logs)
            {
                cleanup_errors.push(format!("saving log: {error}"));
            }
            if let Err(error) = engine.run(
                &["rm".into(), "-f".into(), name.clone()],
                Duration::from_secs(30),
            ) {
                cleanup_errors.push(error.to_string());
            }
        }
    }
    if network_created
        && let Err(error) = engine.run(
            &["network".into(), "rm".into(), network],
            Duration::from_secs(30),
        )
    {
        cleanup_errors.push(error.to_string());
    }
    let report = json!({"ok":outcome.is_ok() && cleanup_errors.is_empty(),
        "source_sha256":env!("MIRKET_SOURCE_SHA256"),"images":state.images,
        "result":outcome.as_ref().ok(), "error":outcome.as_ref().err().map(|e| format!("{e:#}")),
        "cleanup_errors":cleanup_errors,
        "scope":"Fresh isolated database, internal network, tmpfs data; no host ports or saved volumes. Not a production data migration test."});
    fs::write(
        output.join("result.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    ensure!(
        report["ok"] == true,
        "backend check failed; inspect {}",
        output.display()
    );
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn readiness_requires_actual_profile_not_html_or_arbitrary_json() {
        for value in ["OK", "<html>healthy</html>", "{}", "[]", "{\"id\":null}"] {
            assert!(profile(value).is_err());
        }
        assert!(profile("{\"id\":\"00000000-0000-0000-0000-000000000000\"}").is_ok());
    }
}
