use std::{collections::BTreeMap, sync::OnceLock};

use include_dir::{Dir, DirEntry, File, include_dir};
use jsonschema::{Draft, Resource};
use serde_json::Value;

use crate::{
    AppDataDoc, AppDataError, AppDataParams, LATEST_APP_DATA_VERSION, SchemaVersion,
    ValidationResult,
};

static SCHEMAS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/schemas");
static SCHEMA_RESOURCES: OnceLock<BTreeMap<String, Value>> = OnceLock::new();
static ROOT_SCHEMAS: OnceLock<BTreeMap<String, Value>> = OnceLock::new();

const SCHEMA_BASE_URI: &str = "https://cowswap.exchange/schemas/app-data/";

/// Builds a canonical app-data document from typed parameters.
///
/// The typed `signer` and `flashloan` sub-fields on [`AppDataParams`] are
/// merged into the nested `metadata` object in their reviewed camelCase
/// positions before the document is sealed, so the document carries the
/// same wire shape whether the caller supplied the typed fields directly
/// or folded them through the open-ended metadata map.
///
/// # Panics
///
/// Panics only if the typed `flashloan` sub-field ever stops serializing to
/// JSON — which cannot happen for values produced through the public
/// constructors.
#[must_use]
pub fn generate_app_data_doc(params: AppDataParams) -> AppDataDoc {
    // SAFETY: typed metadata values are SDK-owned serde shapes; serialization
    // failure means a broken crate invariant rather than caller input.
    let metadata = params
        .metadata_wire_value()
        .expect("typed flashloan metadata must remain serializable");
    let mut doc = serde_json::Map::new();
    doc.insert(
        "appCode".to_string(),
        Value::String(params.app_code.unwrap_or_else(|| "CoW Swap".to_string())),
    );
    if let Some(environment) = params.environment {
        doc.insert("environment".to_string(), Value::String(environment));
    }
    doc.insert("metadata".to_string(), metadata);
    doc.insert(
        "version".to_string(),
        Value::String(LATEST_APP_DATA_VERSION.to_string()),
    );
    Value::Object(doc)
}

/// Returns the bundled app-data schema for `version`.
///
/// # Errors
///
/// Returns [`AppDataError::InvalidSchemaVersion`] when `version` is not
/// `<major>.<minor>.<patch>`, or [`AppDataError::UnknownSchemaVersion`] when
/// the version is valid but no bundled schema exists for it.
///
/// # Panics
///
/// Panics only if the embedded schema bundle stops following the committed
/// URI, file-name, and JSON-validity invariants validated with the crate.
pub fn get_app_data_schema(version: &str) -> Result<AppDataDoc, AppDataError> {
    let version = SchemaVersion::new(version)?;
    root_schemas()
        .get(version.as_str())
        .cloned()
        .map_or_else(|| Err(AppDataError::UnknownSchemaVersion(version)), Ok)
}

/// Validates an app-data document against the bundled JSON schema set.
///
/// # Panics
///
/// Panics only if the embedded schema bundle stops following the committed
/// URI, file-name, and JSON-validity invariants validated with the crate.
#[must_use]
pub fn validate_app_data_doc(app_data_doc: &AppDataDoc) -> ValidationResult {
    match validate_app_data_doc_inner(app_data_doc) {
        Ok(()) => ValidationResult {
            success: true,
            errors: None,
        },
        Err(err) => {
            let errors = match &err {
                AppDataError::Schema { message, .. } => message.as_inner().clone(),
                _ => err.to_string(),
            };
            ValidationResult {
                success: false,
                errors: Some(errors.into()),
            }
        }
    }
}

/// Extracts the schema version string from an app-data document.
///
/// # Errors
///
/// Returns [`AppDataError::MissingSchemaVersion`] when the document does not
/// contain a string-valued `version` field.
pub fn extract_schema_version(app_data_doc: &AppDataDoc) -> Result<&str, AppDataError> {
    app_data_doc
        .get("version")
        .and_then(Value::as_str)
        .ok_or(AppDataError::MissingSchemaVersion)
}

fn validate_app_data_doc_inner(app_data_doc: &AppDataDoc) -> Result<(), AppDataError> {
    let version = extract_schema_version(app_data_doc)?;
    let schema = get_app_data_schema(version)?;

    let mut options = jsonschema::options().with_draft(Draft::Draft7);
    for (uri, resource) in schema_resources() {
        options = options.with_resource(uri.clone(), Resource::from_contents(resource.clone()));
    }

    let validator = options.build(&schema).map_err(|err| {
        let message = render_validation_error(&err);
        AppDataError::Schema {
            message: message.into(),
            source: Box::new(err.to_owned()),
        }
    })?;

    let mut errors = validator.iter_errors(app_data_doc);
    if let Some(first) = errors.next() {
        let mut rendered = render_validation_error(&first);
        for error in errors {
            rendered.push_str("; ");
            rendered.push_str(&render_validation_error(&error));
        }
        return Err(AppDataError::Schema {
            message: rendered.into(),
            source: Box::new(first.to_owned()),
        });
    }

    Ok(())
}

fn render_validation_error(error: &jsonschema::ValidationError<'_>) -> String {
    let path = error.instance_path().to_string();
    if path.is_empty() {
        format!("data {error}")
    } else {
        format!("data{path} {error}")
    }
}

fn schema_resources() -> &'static BTreeMap<String, Value> {
    SCHEMA_RESOURCES.get_or_init(|| {
        let mut resources = BTreeMap::new();
        collect_files(&SCHEMAS_DIR, "", &mut resources);
        resources
    })
}

/// Returns the embedded root app-data schemas keyed by version.
///
/// # Panics
///
/// Panics only if an embedded schema resource URI was not assembled under the
/// crate-owned schema base URI.
fn root_schemas() -> &'static BTreeMap<String, Value> {
    ROOT_SCHEMAS.get_or_init(|| {
        let mut schemas = BTreeMap::new();
        for (uri, resource) in schema_resources() {
            let relative = uri
                .strip_prefix(SCHEMA_BASE_URI)
                // SAFETY: every schema resource URI is assembled from
                // SCHEMA_BASE_URI inside collect_file.
                .expect("schema URIs are always rooted under SCHEMA_BASE_URI");
            if let Some(version) = relative
                .strip_prefix('v')
                .and_then(|rest| rest.strip_suffix(".json"))
            {
                schemas.insert(version.to_string(), resource.clone());
            }
        }
        schemas
    })
}

/// Recursively collects embedded app-data schema resources.
///
/// # Panics
///
/// Panics only if `include_dir!` yields an embedded directory entry without a
/// stable file name.
fn collect_files(dir: &Dir<'_>, prefix: &str, resources: &mut BTreeMap<String, Value>) {
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(child) => {
                let child_prefix = if prefix.is_empty() {
                    child.path().to_string_lossy().replace('\\', "/")
                } else {
                    format!(
                        "{prefix}/{}",
                        child
                            .path()
                            .file_name()
                            // SAFETY: include_dir! only yields directory entries
                            // with stable embedded names from the crate bundle.
                            .expect("embedded dir has file name")
                            .to_string_lossy()
                    )
                };
                collect_files(child, &child_prefix, resources);
            }
            DirEntry::File(file) => collect_file(file, prefix, resources),
        }
    }
}

/// Collects one embedded app-data schema resource.
///
/// # Panics
///
/// Panics only if `include_dir!` yields a file without a name, or if a committed
/// embedded schema file stops being valid JSON.
fn collect_file(file: &File<'_>, prefix: &str, resources: &mut BTreeMap<String, Value>) {
    let file_name = file
        .path()
        .file_name()
        // SAFETY: include_dir! only yields file entries with stable embedded
        // names from the crate bundle.
        .expect("embedded file has file name")
        .to_string_lossy();
    let relative = if prefix.is_empty() {
        file_name.to_string()
    } else {
        format!("{prefix}/{file_name}")
    };
    let uri = format!("{SCHEMA_BASE_URI}{relative}");
    // SAFETY: committed schema files are validated as JSON by tests and by the
    // policy-maintainer panic allowlist gate.
    let resource: Value =
        serde_json::from_slice(file.contents()).expect("embedded schema json is valid");
    resources.insert(uri, resource);
}
