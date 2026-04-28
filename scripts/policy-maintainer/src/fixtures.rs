use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::de::DeserializeOwned;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FixtureError {
    #[error("failed to read fixture {path}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse YAML fixture {path}")]
    ParseYaml {
        path: PathBuf,
        #[source]
        source: serde_norway::Error,
    },
    #[error("failed to parse JSON fixture {path}")]
    ParseJson {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

pub fn load_yaml<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T, FixtureError> {
    let path = path.as_ref();
    let content = fs::read_to_string(path).map_err(|source| FixtureError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    serde_norway::from_str(&content).map_err(|source| FixtureError::ParseYaml {
        path: path.to_path_buf(),
        source,
    })
}

pub fn load_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T, FixtureError> {
    let path = path.as_ref();
    let content = fs::read_to_string(path).map_err(|source| FixtureError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&content).map_err(|source| FixtureError::ParseJson {
        path: path.to_path_buf(),
        source,
    })
}

pub fn load_github_config_yaml<T: DeserializeOwned>(
    repo_root: impl AsRef<Path>,
    file_name: impl AsRef<Path>,
) -> Result<T, FixtureError> {
    load_yaml(repo_root.as_ref().join(".github/config").join(file_name))
}

pub fn load_parity_openapi_yaml<T: DeserializeOwned>(
    repo_root: impl AsRef<Path>,
    file_name: impl AsRef<Path>,
) -> Result<T, FixtureError> {
    load_yaml(repo_root.as_ref().join("parity/openapi").join(file_name))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use serde::Deserialize;

    use super::{
        FixtureError, load_github_config_yaml, load_json, load_parity_openapi_yaml, load_yaml,
    };

    #[derive(Debug, Deserialize, PartialEq)]
    struct Sample {
        name: String,
        count: u32,
    }

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(name: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "policy-maintainer-{name}-{}-{nanos}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("temp directory should be created");
            Self { path }
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn load_yaml_parses_typed_fixture() {
        let temp = TempDir::new("yaml");
        let path = temp.path.join("fixture.yaml");
        fs::write(&path, "name: example\ncount: 2\n").unwrap();

        let fixture: Sample = load_yaml(&path).unwrap();

        assert_eq!(
            fixture,
            Sample {
                name: "example".to_owned(),
                count: 2
            }
        );
    }

    #[test]
    fn load_json_parses_typed_fixture() {
        let temp = TempDir::new("json");
        let path = temp.path.join("fixture.json");
        fs::write(&path, r#"{"name":"example","count":3}"#).unwrap();

        let fixture: Sample = load_json(&path).unwrap();

        assert_eq!(
            fixture,
            Sample {
                name: "example".to_owned(),
                count: 3
            }
        );
    }

    #[test]
    fn load_yaml_reports_missing_fixture_path() {
        let temp = TempDir::new("missing");
        let path = temp.path.join("missing.yaml");
        let error = load_yaml::<Sample>(&path).unwrap_err();

        assert!(matches!(error, FixtureError::Read { .. }));
        assert!(error.to_string().contains(path.to_str().unwrap()));
    }

    #[test]
    fn named_yaml_helpers_load_expected_directories() {
        let temp = TempDir::new("named-helpers");
        let github_config = temp.path.join(".github/config");
        let parity_openapi = temp.path.join("parity/openapi");
        fs::create_dir_all(&github_config).unwrap();
        fs::create_dir_all(&parity_openapi).unwrap();
        fs::write(github_config.join("gate.yaml"), "name: github\ncount: 4\n").unwrap();
        fs::write(
            parity_openapi.join("schema.yaml"),
            "name: openapi\ncount: 5\n",
        )
        .unwrap();

        let github_fixture: Sample = load_github_config_yaml(&temp.path, "gate.yaml").unwrap();
        let openapi_fixture: Sample = load_parity_openapi_yaml(&temp.path, "schema.yaml").unwrap();

        assert_eq!(
            github_fixture,
            Sample {
                name: "github".to_owned(),
                count: 4
            }
        );
        assert_eq!(
            openapi_fixture,
            Sample {
                name: "openapi".to_owned(),
                count: 5
            }
        );
    }
}
