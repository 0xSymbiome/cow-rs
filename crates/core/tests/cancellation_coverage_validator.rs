#![allow(
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    reason = "test-only source validator keeps diagnostics and parsing helpers together"
)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use syn::{Expr, ImplItem, Item, Lit, Member, Type, Visibility};

struct Surface {
    type_name: &'static str,
    source_paths: &'static [&'static str],
    table_path: &'static str,
    seed_methods: &'static [&'static str],
}

const SURFACES: &[Surface] = &[
    Surface {
        type_name: "OrderbookApi",
        source_paths: &["crates/orderbook/src/api.rs"],
        table_path: "crates/orderbook/tests/cancellation_composition_contract.rs",
        seed_methods: &["version"],
    },
    Surface {
        type_name: "SubgraphApi",
        source_paths: &["crates/subgraph/src/api.rs"],
        table_path: "crates/subgraph/tests/cancellation_composition_contract.rs",
        seed_methods: &["totals"],
    },
    Surface {
        type_name: "Trading",
        source_paths: &["crates/trading/src/client"],
        table_path: "crates/trading/tests/cancellation_composition_contract.rs",
        seed_methods: &["quote_only"],
    },
];

#[test]
fn cancellation_tables_cover_every_public_async_method() {
    let root = workspace_root();

    for surface in SURFACES {
        let public_async = public_async_methods(&root, surface.source_paths, surface.type_name);
        let table_methods = cancellation_table_methods(&root.join(surface.table_path));
        assert!(
            !table_methods.is_empty(),
            "{} must keep its TESTED_METHODS cancellation table populated",
            surface.type_name,
        );

        let covered = table_methods
            .iter()
            .cloned()
            .chain(surface.seed_methods.iter().map(ToString::to_string))
            .collect::<BTreeSet<_>>();
        let missing = public_async
            .difference(&covered)
            .cloned()
            .collect::<Vec<_>>();
        let stale_rows = table_methods
            .difference(&public_async)
            .cloned()
            .collect::<Vec<_>>();

        assert!(
            missing.is_empty(),
            "{} has public async methods without cancellation coverage rows or seed coverage: {missing:?}",
            surface.type_name,
        );
        assert!(
            stale_rows.is_empty(),
            "{} cancellation table contains rows that no longer match public async methods: {stale_rows:?}",
            surface.type_name,
        );
    }
}

#[test]
fn trading_sdk_source_directory_aggregates_public_async_methods() {
    let root = workspace_root();
    let public_async = public_async_methods(&root, &["crates/trading/src/client"], "Trading");
    let expected = [
        "approve_cow_protocol",
        "cow_protocol_allowance",
        "order",
        "pre_sign_transaction",
        "quote_only",
        "quote_results",
        "offchain_cancel_order",
        "onchain_cancel_order",
        "post_limit_order",
        "post_swap_order",
        "post_swap_order_from_quote",
    ]
    .into_iter()
    .map(ToOwned::to_owned)
    .collect::<BTreeSet<_>>();

    assert_eq!(
        public_async, expected,
        "Trading directory scan must preserve the reviewed cancellation surface",
    );
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("core crate must live under workspace crates directory")
        .to_path_buf()
}

fn source_files(root: &Path, source_paths: &[&str]) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for source_path in source_paths {
        let path = root.join(source_path);
        if path.is_dir() {
            let entries = fs::read_dir(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            for entry in entries {
                let entry = entry.unwrap_or_else(|error| {
                    panic!("failed to read entry under {}: {error}", path.display())
                });
                let entry_path = entry.path();
                if entry_path
                    .extension()
                    .is_some_and(|extension| extension == "rs")
                {
                    files.push(entry_path);
                }
            }
        } else {
            files.push(path);
        }
    }

    files.sort();
    files
}

fn parse_file(path: &Path) -> syn::File {
    let source = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    syn::parse_file(&source).unwrap_or_else(|error| {
        panic!("failed to parse {} as Rust source: {error}", path.display())
    })
}

fn public_async_methods(root: &Path, source_paths: &[&str], type_name: &str) -> BTreeSet<String> {
    source_files(root, source_paths)
        .into_iter()
        .flat_map(|path| parse_file(&path).items)
        .filter_map(|item| match item {
            Item::Impl(item_impl)
                if item_impl.trait_.is_none()
                    && impl_type_matches(&item_impl.self_ty, type_name) =>
            {
                Some(item_impl.items)
            }
            _ => None,
        })
        .flatten()
        .filter_map(|impl_item| match impl_item {
            ImplItem::Fn(method)
                if matches!(method.vis, Visibility::Public(_))
                    && method.sig.asyncness.is_some() =>
            {
                Some(method.sig.ident.to_string())
            }
            _ => None,
        })
        .collect()
}

fn impl_type_matches(self_ty: &Type, type_name: &str) -> bool {
    match self_ty {
        Type::Path(path) if path.qself.is_none() => path
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == type_name),
        _ => false,
    }
}

fn cancellation_table_methods(path: &Path) -> BTreeSet<String> {
    parse_file(path)
        .items
        .into_iter()
        .find_map(|item| match item {
            Item::Const(item_const) if item_const.ident == "TESTED_METHODS" => {
                Some(method_names_from_expr(&item_const.expr))
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("{} must define const TESTED_METHODS", path.display()))
}

fn method_names_from_expr(expr: &Expr) -> BTreeSet<String> {
    let Some(elements) = table_elements(expr) else {
        panic!("TESTED_METHODS must be assigned from an array literal");
    };

    elements
        .iter()
        .filter_map(|element| match element {
            Expr::Struct(structure) => structure.fields.iter().find_map(|field| {
                if matches!(&field.member, Member::Named(ident) if ident == "method_name")
                    && let Expr::Lit(literal) = &field.expr
                    && let Lit::Str(value) = &literal.lit
                {
                    return Some(value.value());
                }
                None
            }),
            _ => None,
        })
        .collect()
}

fn table_elements(expr: &Expr) -> Option<&syn::punctuated::Punctuated<Expr, syn::Token![,]>> {
    match expr {
        Expr::Reference(reference) => table_elements(&reference.expr),
        Expr::Array(array) => Some(&array.elems),
        _ => None,
    }
}
