//! One fixed Chrome for Testing runtime. The archive is acquired by the
//! first-party installer; the complete extracted tree remains part of the
//! video release identity and setup transaction.

use super::*;

const MAC_SELECTION: &[u8] =
    include_bytes!("../../packaging/admission/m3-browser-cft-macos-arm64.json");
const WINDOWS_SELECTION: &[u8] =
    include_bytes!("../../packaging/admission/m3-browser-cft-windows-x64.json");
const MAC_SELECTION_SHA256: &str =
    "351f1921ac62f03ee2054ee6d3a3036dd054f2205c4dabe716729a259f28d84b";
const WINDOWS_SELECTION_SHA256: &str =
    "47ee79c7d649d333c01aea0510dff14337c5aa752dd89119fa185d398cb80185";
const MAC_EXECUTABLE: &str = "runtime/browser/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing";
const WINDOWS_EXECUTABLE: &str = "runtime/browser/chrome-win64/chrome.exe";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Selection {
    schema: u32,
    component: String,
    vendor: String,
    version: String,
    target: String,
    archive: Archive,
    executable: String,
    file_count: usize,
    link_count: usize,
    regular_bytes: u64,
    files: Vec<SelectedBrowserFile>,
    links: Vec<ReleaseLink>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Archive {
    name: String,
    url: String,
    bytes: u64,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectedBrowserFile {
    path: String,
    bytes: u64,
    sha256: String,
    mode: u32,
}

pub(super) fn executable(target: &str) -> &'static str {
    if target == MAC_TARGET {
        MAC_EXECUTABLE
    } else {
        WINDOWS_EXECUTABLE
    }
}

pub(super) fn extend(result: &mut Layout, target: &str) -> Result<(), DistributionError> {
    let (source, source_digest, expected_target, archive_name, archive_size, archive_digest) =
        match target {
            MAC_TARGET => (
                MAC_SELECTION,
                MAC_SELECTION_SHA256,
                MAC_TARGET,
                "chrome-mac-arm64.zip",
                194_932_023,
                "ad7f2b49a4dc679af286b1b1b386670d93ca7868079f69012d21ba0ac8293df3",
            ),
            WINDOWS_X64_TARGET | WINDOWS_ARM64_TARGET => (
                WINDOWS_SELECTION,
                WINDOWS_SELECTION_SHA256,
                WINDOWS_X64_TARGET,
                "chrome-win64.zip",
                209_154_801,
                "a4e0d57245e167e98a9d465af1e3774d2b4ec9f04e483d19bcd97a6f63cf46dd",
            ),
            _ => return Err(DistributionError::UnsupportedTarget),
        };
    if hex(&Sha256::digest(source)) != source_digest {
        return Err(DistributionError::InvalidManifest);
    }
    let selected: Selection =
        serde_json::from_slice(source).map_err(|_| DistributionError::InvalidManifest)?;
    if selected.schema != 1
        || selected.component != "browser"
        || selected.vendor != "Google Chrome for Testing"
        || selected.version != "156.0.8078.47"
        || selected.target != expected_target
        || selected.archive.name != archive_name
        || selected.archive.bytes != archive_size
        || selected.archive.sha256 != archive_digest
        || selected.archive.url
            != format!(
                "https://storage.googleapis.com/chrome-for-testing-public/156.0.8078.47/{}/{}",
                if target == MAC_TARGET {
                    "mac-arm64"
                } else {
                    "win64"
                },
                archive_name
            )
        || selected.executable != executable(target)
        || selected.file_count != selected.files.len()
        || selected.link_count != selected.links.len()
        || selected.file_count > 500
        || selected.link_count > 8
    {
        return Err(DistributionError::InvalidManifest);
    }
    let mut total = 0_u64;
    let mut seen = BTreeSet::new();
    for file in selected.files {
        if !safe_relative(&file.path)
            || !file.path.starts_with("runtime/browser/")
            || !valid_digest(&file.sha256)
            || file.bytes > 512 * 1024 * 1024
            || !matches!(file.mode, 0o600 | 0o700)
            || !seen.insert(file.path.clone())
        {
            return Err(DistributionError::InvalidManifest);
        }
        total = total
            .checked_add(file.bytes)
            .ok_or(DistributionError::TooLarge)?;
        insert_pinned(result, file.path, file.bytes, file.sha256, file.mode)?;
    }
    if total != selected.regular_bytes
        || !result.files.contains_key(executable(target))
        || result.files[executable(target)].mode != 0o700
    {
        return Err(DistributionError::InvalidManifest);
    }
    for link in selected.links {
        if !safe_relative(&link.relative_path)
            || !link.relative_path.starts_with("runtime/browser/")
            || !safe_relative(&link.target)
            || !seen.insert(link.relative_path.clone())
            || result
                .links
                .insert(link.relative_path.clone(), link)
                .is_some()
        {
            return Err(DistributionError::InvalidManifest);
        }
    }

    Ok(())
}

fn insert_pinned(
    result: &mut Layout,
    path: String,
    bytes: u64,
    sha256: String,
    mode: u32,
) -> Result<(), DistributionError> {
    if result
        .files
        .insert(
            path,
            ExpectedFile {
                mode,
                pin: Some((bytes, sha256)),
            },
        )
        .is_some()
    {
        return Err(DistributionError::InvalidManifest);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_chrome_inventory_composes_for_selected_targets() {
        for target in [MAC_TARGET, WINDOWS_X64_TARGET, WINDOWS_ARM64_TARGET] {
            let mut base = layout(target, &[]).unwrap();
            extend(&mut base, target).unwrap();
            finish_layout(&mut base, target, &[ReleaseComponent::Video]).unwrap();
            assert_eq!(base.files[executable(target)].mode, 0o700);
            assert_eq!(base.links.len(), usize::from(target == MAC_TARGET) * 5);
            assert!(base.capacity.payload_bytes > 150_000_000);
        }
    }
}
