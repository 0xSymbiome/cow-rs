use std::path::{Path, PathBuf};

use syn::{Attribute, Item, Visibility};

#[test]
fn every_public_struct_and_enum_is_non_exhaustive() {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut missing = Vec::new();

    for file in rust_files(&src) {
        let text = std::fs::read_to_string(&file)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", file.display()));
        let syntax = syn::parse_file(&text)
            .unwrap_or_else(|error| panic!("failed to parse {}: {error}", file.display()));

        for item in syntax.items {
            match item {
                Item::Struct(item) if is_public(&item.vis) => {
                    if !has_non_exhaustive(&item.attrs) {
                        missing.push(format!("{}::{}", file.display(), item.ident));
                    }
                }
                Item::Enum(item) if is_public(&item.vis) => {
                    if !has_non_exhaustive(&item.attrs) {
                        missing.push(format!("{}::{}", file.display(), item.ident));
                    }
                }
                _ => {}
            }
        }
    }

    assert!(
        missing.is_empty(),
        "public structs/enums missing #[non_exhaustive]: {missing:#?}"
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

const fn is_public(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

fn has_non_exhaustive(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(|attr| attr.path().is_ident("non_exhaustive"))
}
