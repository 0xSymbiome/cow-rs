use std::path::{Path, PathBuf};

const FORBIDDEN: &[&str] = &[
    ".unwrap(",
    ".expect(",
    "panic!(",
    "unreachable!(",
    "todo!(",
    "unimplemented!(",
];

#[test]
fn shipped_source_has_no_panic_bearing_call_sites() {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut hits = Vec::new();
    for file in rust_files(&src) {
        let text = std::fs::read_to_string(&file)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", file.display()));
        for (line_index, line) in text.lines().enumerate() {
            for pattern in FORBIDDEN {
                if line.contains(pattern) {
                    hits.push(format!(
                        "{}:{} contains {pattern}",
                        file.display(),
                        line_index + 1
                    ));
                }
            }
        }
    }

    assert!(
        hits.is_empty(),
        "panic-bearing source call sites outside allowlist: {hits:#?}"
    );
}

fn rust_files(root: &Path) -> Vec<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(path) = stack.pop() {
        for entry in std::fs::read_dir(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        {
            let entry = entry.expect("directory entry is readable");
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }
    files
}
