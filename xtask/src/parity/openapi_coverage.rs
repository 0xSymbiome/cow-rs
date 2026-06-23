use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::Args;
use serde::{Deserialize, Serialize};
use serde_norway::Value;
use syn::{Field, Fields, GenericArgument, Item, PathArguments, Type};

const DEFAULT_COVERAGE_MANIFEST: &str = "parity/openapi/coverage.yaml";
const DEFAULT_OPENAPI: &str = "parity/openapi/services-orderbook.yml";

#[derive(Debug, Args)]
pub struct OpenApiCoverageArgs {
    #[arg(long, default_value = DEFAULT_COVERAGE_MANIFEST)]
    coverage: PathBuf,
    #[arg(long, default_value = DEFAULT_OPENAPI)]
    openapi: PathBuf,
    #[arg(long)]
    schema: Option<String>,
    #[arg(long)]
    rust_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CoverageManifest {
    version: u32,
    dtos: Vec<CoverageEntry>,
    #[serde(default)]
    excluded_schemas: Vec<ExcludedSchemas>,
}

/// A reason-annotated bucket of top-level spec schemas intentionally not
/// enrolled as `dtos`. Every `components.schemas.*` must be either a `dtos`
/// entry or listed here, so a newly vendored schema fails the ratchet until a
/// maintainer decides.
#[derive(Debug, Deserialize)]
struct ExcludedSchemas {
    reason: String,
    schemas: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CoverageEntry {
    schema: String,
    rust_type: String,
    required_fields: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SchemaInventory {
    schema: String,
    #[serde(rename = "allOf")]
    all_of: Vec<String>,
    #[serde(rename = "oneOf")]
    one_of: Vec<String>,
    expanded_required: Vec<String>,
    expanded_fields: Vec<InventoryField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InventoryField {
    name: String,
    rust_name: String,
    #[serde(rename = "type")]
    field_type: InventoryType,
    required: bool,
    nullable: bool,
    default: Option<Value>,
    source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum InventoryType {
    Scalar { scalar: String },
    Ref { ref_path: String },
    Array { items: Box<Self> },
    Object,
    Unknown,
}

#[derive(Debug, Serialize)]
struct ValidationReport {
    status: &'static str,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Serialize)]
struct Diagnostic {
    schema: String,
    rust_type: String,
    field: Option<String>,
    kind: &'static str,
    message: String,
}

pub fn run(args: &OpenApiCoverageArgs) -> Result<()> {
    let manifest = load_manifest(&args.coverage)?;
    let openapi = load_yaml(&args.openapi)?;
    let selected = selected_entries(&manifest, args.schema.as_deref(), args.rust_type.as_deref())?;
    let mut diagnostics = Vec::new();

    for entry in &selected {
        // The per-schema inventory is expanded in memory from the vendored
        // OpenAPI document; it is not a committed artifact, so there is no
        // generated cache to keep in lockstep with the spec.
        let inventory = match build_inventory(&openapi, &entry.schema) {
            Ok(inventory) => inventory,
            Err(error) => {
                diagnostics.push(Diagnostic {
                    schema: entry.schema.clone(),
                    rust_type: entry.rust_type.clone(),
                    field: None,
                    kind: "inventory_build_error",
                    message: format!("{error:#}"),
                });
                continue;
            }
        };

        let rust_struct = match load_rust_struct(&entry.rust_type) {
            Ok(rust_struct) => rust_struct,
            Err(error) => {
                diagnostics.push(Diagnostic {
                    schema: entry.schema.clone(),
                    rust_type: entry.rust_type.clone(),
                    field: None,
                    kind: "rust_type_read_error",
                    message: format!("{error:#}"),
                });
                continue;
            }
        };

        validate_required_fields(entry, &inventory, &mut diagnostics);
        validate_inventory_fields(entry, &inventory, &rust_struct, &mut diagnostics);
    }

    // Spec-completeness only makes sense over the whole manifest, not a
    // single `--schema`/`--rust_type` slice.
    if args.schema.is_none() && args.rust_type.is_none() {
        validate_schema_completeness(&manifest, &openapi, &mut diagnostics)?;
    }

    if diagnostics.is_empty() {
        println!(
            "validated OpenAPI coverage for {} DTO entries",
            selected.len()
        );
        Ok(())
    } else {
        let report = ValidationReport {
            status: "failed",
            diagnostics,
        };
        eprintln!(
            "{}",
            serde_norway::to_string(&report).context("failed to serialize diagnostics")?
        );
        bail!("openapi coverage validation failed")
    }
}

/// The coverage ratchet: every top-level `components.schemas.*` in the vendored
/// spec must be enrolled as a `dtos` entry or recorded in `excluded_schemas`
/// with a reason. A newly vendored schema that is neither fails closed, forcing
/// a conscious model-it-or-exclude-it decision rather than silent omission.
fn validate_schema_completeness(
    manifest: &CoverageManifest,
    openapi: &Value,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
    let schemas = component_schema_names(openapi)?;
    let covered: BTreeSet<&str> = manifest
        .dtos
        .iter()
        .filter_map(|entry| entry.schema.strip_prefix("components.schemas."))
        .filter(|rest| !rest.contains('.'))
        .collect();

    let mut excluded: BTreeSet<&str> = BTreeSet::new();
    for bucket in &manifest.excluded_schemas {
        if bucket.reason.trim().is_empty() {
            diagnostics.push(schema_diagnostic(
                String::new(),
                "excluded_without_reason",
                format!(
                    "excluded_schemas bucket {:?} carries no reason",
                    bucket.schemas
                ),
            ));
        }
        for schema in &bucket.schemas {
            if !schemas.contains(schema.as_str()) {
                diagnostics.push(schema_diagnostic(
                    schema.clone(),
                    "stale_excluded_schema",
                    format!("excluded schema `{schema}` is not present in components.schemas"),
                ));
            }
            if covered.contains(schema.as_str()) {
                diagnostics.push(schema_diagnostic(
                    schema.clone(),
                    "excluded_and_covered",
                    format!("schema `{schema}` is both enrolled as a dto and excluded"),
                ));
            }
            excluded.insert(schema.as_str());
        }
    }

    for schema in &schemas {
        if !covered.contains(schema.as_str()) && !excluded.contains(schema.as_str()) {
            diagnostics.push(schema_diagnostic(
                schema.clone(),
                "uncovered_schema",
                format!(
                    "schema `{schema}` is neither enrolled as a dto nor recorded in excluded_schemas; model it or record why not"
                ),
            ));
        }
    }
    Ok(())
}

/// The top-level schema names under `components.schemas`.
fn component_schema_names(openapi: &Value) -> Result<BTreeSet<String>> {
    let schemas = resolve_schema(openapi, "components.schemas")?;
    let mapping = schemas
        .as_mapping()
        .context("components.schemas is not a mapping")?;
    Ok(mapping
        .keys()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect())
}

/// A schema-level diagnostic, with no Rust type or field.
const fn schema_diagnostic(schema: String, kind: &'static str, message: String) -> Diagnostic {
    Diagnostic {
        schema,
        rust_type: String::new(),
        field: None,
        kind,
        message,
    }
}

fn load_manifest(path: &Path) -> Result<CoverageManifest> {
    let manifest: CoverageManifest = serde_norway::from_str(
        &fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", path.display()))?;
    if manifest.version != 1 {
        bail!("expected OpenAPI coverage manifest version 1");
    }
    if manifest.dtos.is_empty() {
        bail!("OpenAPI coverage manifest has no dto entries");
    }
    Ok(manifest)
}

fn selected_entries<'a>(
    manifest: &'a CoverageManifest,
    schema: Option<&str>,
    rust_type: Option<&str>,
) -> Result<Vec<&'a CoverageEntry>> {
    let selected = manifest
        .dtos
        .iter()
        .filter(|entry| schema.is_none_or(|schema| entry.schema == schema))
        .filter(|entry| rust_type.is_none_or(|rust_type| entry.rust_type == rust_type))
        .collect::<Vec<_>>();
    if selected.is_empty() {
        bail!("no coverage entries matched the requested filters");
    }
    Ok(selected)
}

fn load_yaml(path: &Path) -> Result<Value> {
    serde_norway::from_str(
        &fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", path.display()))
}

fn build_inventory(openapi: &Value, schema: &str) -> Result<SchemaInventory> {
    let mut inventory = SchemaInventory {
        schema: schema.to_string(),
        all_of: Vec::new(),
        one_of: Vec::new(),
        expanded_required: Vec::new(),
        expanded_fields: Vec::new(),
    };
    let mut fields = BTreeMap::new();
    let schema_value = resolve_schema(openapi, schema)?;
    expand_schema(
        openapi,
        schema_value,
        schema,
        false,
        &mut inventory,
        &mut fields,
    )?;
    inventory.expanded_required = fields
        .values()
        .filter(|field| field.required)
        .map(|field| field.name.clone())
        .collect();
    inventory.expanded_fields = fields.into_values().collect();
    Ok(inventory)
}

fn expand_schema(
    openapi: &Value,
    schema: &Value,
    source: &str,
    force_optional: bool,
    inventory: &mut SchemaInventory,
    fields: &mut BTreeMap<String, InventoryField>,
) -> Result<()> {
    if let Some(reference) = schema_ref(schema) {
        let ref_schema = ref_to_schema_path(reference)?;
        let resolved = resolve_schema(openapi, &ref_schema)?;
        return expand_schema(
            openapi,
            resolved,
            &ref_schema,
            force_optional,
            inventory,
            fields,
        );
    }

    if let Some(branches) = value_array(schema, "allOf") {
        for branch in branches {
            let branch_source = branch_source(branch, source)?;
            inventory.all_of.push(branch_source.clone());
            expand_schema(
                openapi,
                branch,
                &branch_source,
                force_optional,
                inventory,
                fields,
            )?;
        }
    }

    if let Some(branches) = value_array(schema, "oneOf") {
        for branch in branches {
            let branch_source = branch_source(branch, source)?;
            inventory.one_of.push(branch_source.clone());
            expand_schema(openapi, branch, &branch_source, true, inventory, fields)?;
        }
    }

    let required = required_fields(schema);
    if let Some(properties) = value_mapping(schema, "properties") {
        for (name, property) in properties {
            // Deprecated wire fields are excluded from the coverage surface: the
            // SDK does not mirror fields the upstream spec marks for removal.
            if bool_value(property, "deprecated") {
                continue;
            }
            let required = !force_optional && required.contains(name.as_str());
            let field = InventoryField {
                name: name.clone(),
                rust_name: camel_to_snake(&name),
                field_type: inventory_type(property),
                required,
                nullable: bool_value(property, "nullable"),
                default: mapping_get(property, "default").cloned(),
                source: source.to_string(),
            };
            merge_field(fields, field);
        }
    }

    Ok(())
}

fn merge_field(fields: &mut BTreeMap<String, InventoryField>, field: InventoryField) {
    fields
        .entry(field.name.clone())
        .and_modify(|existing| {
            existing.required |= field.required;
            existing.nullable |= field.nullable;
            if existing.default.is_none() {
                existing.default.clone_from(&field.default);
            }
        })
        .or_insert(field);
}

fn branch_source(branch: &Value, fallback: &str) -> Result<String> {
    Ok(schema_ref(branch)
        .map(ref_to_schema_path)
        .transpose()?
        .unwrap_or_else(|| format!("{fallback}.inline")))
}

fn resolve_schema<'a>(openapi: &'a Value, schema: &str) -> Result<&'a Value> {
    let mut current = openapi;
    for part in schema.split('.') {
        current = mapping_get(current, part)
            .with_context(|| format!("schema path {schema} is missing segment {part}"))?;
    }
    Ok(current)
}

fn schema_ref(value: &Value) -> Option<&str> {
    mapping_get(value, "$ref").and_then(Value::as_str)
}

fn ref_to_schema_path(reference: &str) -> Result<String> {
    let rest = reference
        .strip_prefix("#/")
        .with_context(|| format!("unsupported OpenAPI ref {reference}"))?;
    Ok(rest.replace('/', "."))
}

fn inventory_type(value: &Value) -> InventoryType {
    if let Some(reference) = schema_ref(value) {
        return InventoryType::Ref {
            ref_path: ref_to_schema_path(reference).unwrap_or_else(|_| reference.to_string()),
        };
    }
    if let Some(items) = mapping_get(value, "items") {
        return InventoryType::Array {
            items: Box::new(inventory_type(items)),
        };
    }
    match mapping_get(value, "type").and_then(Value::as_str) {
        Some("array") => InventoryType::Array {
            items: Box::new(
                mapping_get(value, "items").map_or(InventoryType::Unknown, inventory_type),
            ),
        },
        Some("object") => InventoryType::Object,
        Some(scalar) => InventoryType::Scalar {
            scalar: scalar.to_string(),
        },
        None if value_array(value, "oneOf").is_some() => InventoryType::Object,
        None if value_array(value, "allOf").is_some() => InventoryType::Object,
        None => InventoryType::Unknown,
    }
}

fn required_fields(value: &Value) -> BTreeSet<String> {
    value_array(value, "required")
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect()
}

fn value_array<'a>(value: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    mapping_get(value, key).and_then(Value::as_sequence)
}

fn value_mapping<'a>(value: &'a Value, key: &str) -> Option<BTreeMap<String, &'a Value>> {
    mapping_get(value, key)
        .and_then(Value::as_mapping)
        .map(|map| {
            map.iter()
                .filter_map(|(key, value)| key.as_str().map(|key| (key.to_string(), value)))
                .collect()
        })
}

fn mapping_get<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value.as_mapping()?.get(Value::String(key.to_string()))
}

fn bool_value(value: &Value, key: &str) -> bool {
    mapping_get(value, key)
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn validate_inventory_fields(
    entry: &CoverageEntry,
    inventory: &SchemaInventory,
    rust_struct: &RustStruct,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for inventory_field in &inventory.expanded_fields {
        let expected_name = camel_to_snake(&inventory_field.name);
        let Some(rust_field) = rust_struct.fields.get(&expected_name) else {
            diagnostics.push(Diagnostic {
                schema: entry.schema.clone(),
                rust_type: entry.rust_type.clone(),
                field: Some(inventory_field.name.clone()),
                kind: "missing_field",
                message: format!(
                    "expected Rust field `{expected_name}` for OpenAPI field `{}`",
                    inventory_field.name
                ),
            });
            continue;
        };

        let rust_optional = option_inner(&rust_field.ty).is_some();
        let expected_optional = expects_option(inventory_field);
        if rust_optional != expected_optional {
            diagnostics.push(Diagnostic {
                schema: entry.schema.clone(),
                rust_type: entry.rust_type.clone(),
                field: Some(inventory_field.name.clone()),
                kind: "optionality_mismatch",
                message: format!(
                    "OpenAPI field `{}` expects optional={}, Rust field `{expected_name}` optional={rust_optional}",
                    inventory_field.name, expected_optional
                ),
            });
        }

        if expects_serde_default_skip(inventory_field)
            && !(has_serde_default(rust_field) && has_serde_skip_option(rust_field))
        {
            diagnostics.push(Diagnostic {
                schema: entry.schema.clone(),
                rust_type: entry.rust_type.clone(),
                field: Some(inventory_field.name.clone()),
                kind: "serde_optional_mismatch",
                message: format!(
                    "optional non-null OpenAPI field `{}` requires #[serde(default, skip_serializing_if = \"Option::is_none\")]",
                    inventory_field.name
                ),
            });
        }

        if inventory_field.default.is_some() && !has_serde_default(rust_field) {
            diagnostics.push(Diagnostic {
                schema: entry.schema.clone(),
                rust_type: entry.rust_type.clone(),
                field: Some(inventory_field.name.clone()),
                kind: "serde_default_missing",
                message: format!(
                    "OpenAPI field `{}` has a default and requires a serde default on Rust field `{expected_name}`",
                    inventory_field.name
                ),
            });
        }

        let expected_types = expected_rust_types(inventory_field);
        if !expected_types.is_empty() {
            let rust_type = comparable_type(&rust_field.ty);
            if !rust_type.as_deref().is_some_and(|rust_type| {
                expected_types.iter().any(|expected| expected == rust_type)
            }) {
                diagnostics.push(Diagnostic {
                    schema: entry.schema.clone(),
                    rust_type: entry.rust_type.clone(),
                    field: Some(inventory_field.name.clone()),
                    kind: "type_mismatch",
                    message: format!(
                        "OpenAPI field `{}` expects Rust type `{}`, found `{}`",
                        inventory_field.name,
                        expected_types.join(" | "),
                        rust_type.unwrap_or_else(|| "<unsupported>".to_string())
                    ),
                });
            }
        }
    }
}

fn validate_required_fields(
    entry: &CoverageEntry,
    inventory: &SchemaInventory,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let expected = entry
        .required_fields
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let observed = inventory
        .expanded_required
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if expected != observed {
        diagnostics.push(Diagnostic {
            schema: entry.schema.clone(),
            rust_type: entry.rust_type.clone(),
            field: None,
            kind: "required_fields_mismatch",
            message: format!(
                "coverage.yaml required_fields {:?} must match inventory expanded_required {:?}",
                entry.required_fields, inventory.expanded_required
            ),
        });
    }
}

const fn expects_option(field: &InventoryField) -> bool {
    if field.required && !field.nullable {
        return false;
    }
    if field.default.is_some() {
        return false;
    }
    true
}

const fn expects_serde_default_skip(field: &InventoryField) -> bool {
    !field.required && !field.nullable && field.default.is_none()
}

fn expected_rust_types(field: &InventoryField) -> Vec<String> {
    match &field.field_type {
        InventoryType::Scalar { scalar } => match scalar.as_str() {
            "string" if is_amount_like_field(&field.name) => {
                vec!["String".to_string(), "Amount".to_string()]
            }
            "string" => vec!["String".to_string()],
            "boolean" => vec!["bool".to_string()],
            "integer" => [
                "i8", "i16", "i32", "i64", "isize", "u8", "u16", "u32", "u64", "usize",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            "number" => vec!["f32".to_string(), "f64".to_string()],
            _ => Vec::new(),
        },
        InventoryType::Ref { ref_path } => ref_path
            .rsplit('.')
            .next()
            .map(openapi_ref_rust_types)
            .unwrap_or_default(),
        InventoryType::Array { items } => {
            let item_field = InventoryField {
                name: field.name.clone(),
                rust_name: field.rust_name.clone(),
                field_type: (**items).clone(),
                required: field.required,
                nullable: field.nullable,
                default: field.default.clone(),
                source: field.source.clone(),
            };
            expected_rust_types(&item_field)
                .into_iter()
                .map(|item| format!("Vec<{item}>"))
                .collect()
        }
        InventoryType::Object | InventoryType::Unknown => Vec::new(),
    }
}

fn is_amount_like_field(name: &str) -> bool {
    name.ends_with("Amount") || name == "totalSurplus"
}

fn openapi_ref_rust_types(schema_name: &str) -> Vec<String> {
    match schema_name {
        "Address" => vec!["Address".to_string()],
        "AppDataHash" => vec!["AppDataHash".to_string()],
        "BigUint" | "TokenAmount" => vec!["Amount".to_string()],
        "BuyTokenDestination" => vec!["BuyTokenDestination".to_string()],
        "CallData" | "EcdsaSignature" | "PreSignature" | "Signature" => {
            vec!["String".to_string()]
        }
        "OrderClass" => vec!["OrderClass".to_string()],
        "OrderKind" => vec!["OrderKind".to_string()],
        "SellTokenSource" => vec!["SellTokenSource".to_string()],
        "SigningScheme" => vec!["SigningScheme".to_string()],
        "TransactionHash" => vec!["TransactionHash".to_string(), "String".to_string()],
        "UID" => vec!["OrderUid".to_string()],
        other => vec![other.to_string()],
    }
}

struct RustStruct {
    fields: BTreeMap<String, Field>,
}

fn load_rust_struct(rust_type: &str) -> Result<RustStruct> {
    let struct_name = rust_type
        .rsplit("::")
        .next()
        .with_context(|| format!("invalid rust type path {rust_type}"))?;
    let crate_src = crate_src_dir(rust_type)?;
    let files = crate::parity::collect_files(&crate_src, "rs")?;

    for file in files {
        let parsed = syn::parse_file(
            &fs::read_to_string(&file)
                .with_context(|| format!("failed to read {}", file.display()))?,
        )
        .with_context(|| format!("failed to parse {}", file.display()))?;
        if let Some(fields) = find_struct_fields(&parsed.items, struct_name) {
            return Ok(RustStruct { fields });
        }
    }

    bail!(
        "failed to find struct `{struct_name}` under {}",
        crate_src.display()
    )
}

fn crate_src_dir(rust_type: &str) -> Result<PathBuf> {
    let crate_name = rust_type
        .split("::")
        .next()
        .with_context(|| format!("invalid rust type path {rust_type}"))?;
    let crate_dir = match crate_name {
        "cow_sdk_app_data" => "crates/app-data/src",
        "cow_sdk_contracts" => "crates/contracts/src",
        "cow_sdk_core" => "crates/core/src",
        "cow_sdk_orderbook" => "crates/orderbook/src",
        "cow_sdk_sdk" | "cow_sdk" => "crates/sdk/src",
        "cow_sdk_signing" => "crates/signing/src",
        "cow_sdk_subgraph" => "crates/subgraph/src",
        "cow_sdk_trading" => "crates/trading/src",
        other => bail!("unsupported crate prefix `{other}` in rust type `{rust_type}`"),
    };
    Ok(PathBuf::from(crate_dir))
}

fn find_struct_fields(items: &[Item], struct_name: &str) -> Option<BTreeMap<String, Field>> {
    for item in items {
        match item {
            Item::Struct(item_struct) if item_struct.ident == struct_name => {
                let Fields::Named(fields) = &item_struct.fields else {
                    return Some(BTreeMap::new());
                };
                return Some(
                    fields
                        .named
                        .iter()
                        .filter_map(|field| {
                            field
                                .ident
                                .as_ref()
                                .map(|ident| (ident.to_string(), field.clone()))
                        })
                        .collect(),
                );
            }
            Item::Mod(item_mod) => {
                if let Some((_, items)) = &item_mod.content
                    && let Some(fields) = find_struct_fields(items, struct_name)
                {
                    return Some(fields);
                }
            }
            _ => {}
        }
    }
    None
}

fn option_inner(ty: &Type) -> Option<&Type> {
    generic_inner_type(ty, "Option")
}

fn generic_inner_type<'a>(ty: &'a Type, ident: &str) -> Option<&'a Type> {
    let Type::Path(path) = ty else {
        return None;
    };
    let segment = path.path.segments.last()?;
    if segment.ident != ident {
        return None;
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    arguments.args.iter().find_map(|argument| {
        if let GenericArgument::Type(ty) = argument {
            Some(ty)
        } else {
            None
        }
    })
}

fn comparable_type(ty: &Type) -> Option<String> {
    let ty = option_inner(ty).unwrap_or(ty);
    if let Some(inner) = generic_inner_type(ty, "Vec") {
        return comparable_type(inner).map(|inner| format!("Vec<{inner}>"));
    }
    match ty {
        Type::Path(path) => path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        Type::Reference(reference) => comparable_type(&reference.elem),
        _ => None,
    }
}

fn has_serde_default(field: &Field) -> bool {
    serde_attr_contains(field, |meta| meta == "default")
}

fn has_serde_skip_option(field: &Field) -> bool {
    serde_attr_contains(field, |meta| {
        meta == "skip_serializing_if" || meta == "skip_serializing_if = \"Option::is_none\""
    })
}

fn serde_attr_contains(field: &Field, mut predicate: impl FnMut(&str) -> bool) -> bool {
    field.attrs.iter().any(|attr| {
        if !attr.path().is_ident("serde") {
            return false;
        }
        let mut found = false;
        let _ = attr.parse_nested_meta(|meta| {
            if let Some(ident) = meta.path.get_ident() {
                let ident = ident.to_string();
                if predicate(&ident) {
                    found = true;
                }
                if meta.input.peek(syn::Token![=]) {
                    let value = meta.value()?;
                    let literal: syn::LitStr = value.parse()?;
                    let combined = format!("{ident} = {:?}", literal.value());
                    if predicate(&combined) {
                        found = true;
                    }
                }
            }
            Ok(())
        });
        found
    })
}

fn camel_to_snake(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for (idx, ch) in input.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if idx > 0 {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
        } else {
            output.push(ch);
        }
    }
    output
}
