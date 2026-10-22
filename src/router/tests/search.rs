#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn regex_and_nested_gitignore_rules_keep_source_scope_and_bounds() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        std::fs::write(root.join(".gitignore"), "*.log\n!keep.log\nblocked/\n").unwrap();
        std::fs::create_dir(root.join("nested")).unwrap();
        std::fs::create_dir(root.join("blocked")).unwrap();
        std::fs::write(root.join("nested/.gitignore"), "!local.log\n").unwrap();
        for file in ["a.rs", "drop.log", "keep.log", "nested/drop.log", "nested/local.log", "blocked/a.rs"] {
            std::fs::write(root.join(file), "foo_123\nfoo_xyz\n").unwrap();
        }
        let found = search(&root, json!({"query":"foo_[0-9]+","mode":"regex"})).unwrap();
        let paths: Vec<_> = found["matches"].as_array().unwrap().iter().map(|m| m["path"].as_str().unwrap()).collect();
        assert_eq!(paths, ["a.rs", "keep.log", "nested/local.log"]);
        assert_eq!(found["ignore_files"].as_array().unwrap().len(), 2);
        let scoped = search(&root, json!({"query":"foo_123","path":"nested"})).unwrap();
        assert_eq!(scoped["matches"][0]["path"], "nested/local.log");
        assert_eq!(scoped["matches"].as_array().unwrap().len(), 1);
        let blocked = search(&root, json!({"query":"foo_123","path":"blocked/a.rs"})).unwrap();
        assert!(blocked["matches"].as_array().unwrap().is_empty());
        assert!(search(&root, json!({"query":"foo_[0-9]+"})).unwrap()["matches"].as_array().unwrap().is_empty());
        assert!(search(&root, json!({"query":"(?=x)","mode":"regex"})).is_err());
        assert!(search(&root, json!({"query":"x{1000000}","mode":"regex"})).is_err());
        std::fs::write(root.join(".gitignore"), "x".repeat(65537)).unwrap();
        assert!(search(&root, json!({"query":"foo"})).is_err());
        #[cfg(unix)] {
            std::fs::remove_file(root.join(".gitignore")).unwrap();
            std::os::unix::fs::symlink("a.rs", root.join(".gitignore")).unwrap();
            assert!(search(&root, json!({"query":"foo"})).is_err());
        }
    }

    #[test]
    fn aggregate_search_budget_never_reads_one_file_past_the_limit() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        std::fs::write(root.join("000"), "x".repeat(512)).unwrap();
        let block = vec![b'x'; 1024 * 1024];
        for index in 1..=32 {
            std::fs::write(root.join(format!("{index:03}")), &block).unwrap();
        }
        let found = search(&root, json!({"query":"absent"})).unwrap();
        assert_eq!(found["truncated"], true);
        assert_eq!(found["scanned_bytes"], 31 * 1024 * 1024 + 512);
        assert!(found["matches"].as_array().unwrap().is_empty());
    }

    #[test]
    fn bounded_search_retains_source_identity_and_rejects_escapes() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        std::fs::write(root.join("a.rs"), "İlk\nneedle α\nneedle β\n").unwrap();
        std::fs::create_dir(root.join(".git")).unwrap();
        std::fs::write(root.join(".git/secret"), "needle").unwrap();
        let found = search(&root, json!({"query":"needle","limit":1})).unwrap();
        assert_eq!(found["matches"][0]["line"], 2);
        assert_eq!(
            found["matches"][0]["sha256"],
            crate::util::sha256("İlk\nneedle α\nneedle β\n".as_bytes())
        );
        assert_eq!(found["truncated"], true);
        assert!(search(&root, json!({"query":""})).is_err());
        assert!(search(&root, json!({"query":"needle","path":"../outside"})).is_err());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("a.rs", root.join("link")).unwrap();
            let found = search(&root, json!({"query":"needle"})).unwrap();
            assert_eq!(found["matches"].as_array().unwrap().len(), 2);
            assert_eq!(found["skipped"][0]["reason"], "symlink");
        }
    }
}
