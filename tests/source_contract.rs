use std::path::{Path, PathBuf};
fn files(root: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files(&path, out);
        } else {
            out.push(path);
        }
    }
}
#[test]
fn authored_guidance_has_existing_local_links_and_no_detached_helpers() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut sources = vec![
        root.join("README.md"),
        root.join("CONTRIBUTING.md"),
        root.join("AGENTS.md"),
    ];
    for name in ["docs", "skills", "instructions", "setup"] {
        files(&root.join(name), &mut sources);
    }
    let mut broken = vec![];
    for source in sources {
        assert_ne!(
            source.extension().and_then(|s| s.to_str()),
            Some("py"),
            "detached helper {}",
            source.display()
        );
        if source.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let text = std::fs::read_to_string(&source).unwrap();
        for chunk in text.split("](").skip(1) {
            let Some((target, _)) = chunk.split_once(')') else {
                continue;
            };
            let target = target.trim_matches(['<', '>']);
            if target.contains("://") || target.starts_with('#') || target.starts_with("mailto:") {
                continue;
            }
            let target = target.split('#').next().unwrap();
            if target.is_empty() || target.contains('`') {
                continue;
            }
            if !source.parent().unwrap().join(target).exists() {
                broken.push(format!(
                    "{} -> {target}",
                    source.strip_prefix(root).unwrap().display()
                ));
            }
        }
    }
    assert!(
        broken.is_empty(),
        "broken local links:\n{}",
        broken.join("\n")
    );
}
