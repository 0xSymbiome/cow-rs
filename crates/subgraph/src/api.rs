use std::collections::BTreeMap;

use cow_sdk_core::SupportedChainId;
use reqwest::Client;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use crate::{
    error::SubgraphError,
    queries::{LAST_DAYS_VOLUME_QUERY, LAST_HOURS_VOLUME_QUERY, TOTALS_QUERY},
    types::{LastDaysVolumeResponse, LastHoursVolumeResponse, Total, TotalsResponse},
};

const SUBGRAPH_BASE_URL: &str = "https://gateway.thegraph.com/api/";

pub const API_NAME: &str = "CoW Protocol Subgraph";

pub type SubgraphApiBaseUrls = BTreeMap<SupportedChainId, Option<String>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubgraphConfig {
    pub chain_id: SupportedChainId,
    pub base_urls: Option<SubgraphApiBaseUrls>,
}

impl Default for SubgraphConfig {
    fn default() -> Self {
        Self {
            chain_id: SupportedChainId::Mainnet,
            base_urls: None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SubgraphConfigOverride {
    pub chain_id: Option<SupportedChainId>,
    pub base_urls: Option<SubgraphApiBaseUrls>,
}

#[derive(Clone)]
pub struct SubgraphApi {
    client: Client,
    config: SubgraphConfig,
    prod_config: SubgraphApiBaseUrls,
}

impl SubgraphApi {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_config(api_key, SubgraphConfig::default())
    }

    pub fn with_config(api_key: impl Into<String>, config: SubgraphConfig) -> Self {
        let api_key = api_key.into();

        Self {
            client: Client::new(),
            prod_config: build_prod_config(&api_key),
            config,
        }
    }

    pub fn api_name(&self) -> &'static str {
        API_NAME
    }

    pub fn config(&self) -> &SubgraphConfig {
        &self.config
    }

    pub fn prod_config(&self) -> &SubgraphApiBaseUrls {
        &self.prod_config
    }

    pub async fn get_totals(&self) -> Result<Total, SubgraphError> {
        self.get_totals_with_config(SubgraphConfigOverride::default())
            .await
    }

    pub async fn get_totals_with_config(
        &self,
        config_override: SubgraphConfigOverride,
    ) -> Result<Total, SubgraphError> {
        let response: TotalsResponse = self
            .run_query_with_config(TOTALS_QUERY, None, config_override)
            .await?;

        response
            .totals
            .into_iter()
            .next()
            .ok_or(SubgraphError::NoTotalsFound)
    }

    pub async fn get_last_days_volume(
        &self,
        days: u32,
    ) -> Result<LastDaysVolumeResponse, SubgraphError> {
        self.get_last_days_volume_with_config(days, SubgraphConfigOverride::default())
            .await
    }

    pub async fn get_last_days_volume_with_config(
        &self,
        days: u32,
        config_override: SubgraphConfigOverride,
    ) -> Result<LastDaysVolumeResponse, SubgraphError> {
        self.run_query_with_config(
            LAST_DAYS_VOLUME_QUERY,
            Some(json!({ "days": days })),
            config_override,
        )
        .await
    }

    pub async fn get_last_hours_volume(
        &self,
        hours: u32,
    ) -> Result<LastHoursVolumeResponse, SubgraphError> {
        self.get_last_hours_volume_with_config(hours, SubgraphConfigOverride::default())
            .await
    }

    pub async fn get_last_hours_volume_with_config(
        &self,
        hours: u32,
        config_override: SubgraphConfigOverride,
    ) -> Result<LastHoursVolumeResponse, SubgraphError> {
        self.run_query_with_config(
            LAST_HOURS_VOLUME_QUERY,
            Some(json!({ "hours": hours })),
            config_override,
        )
        .await
    }

    pub async fn run_query<T>(
        &self,
        query: &str,
        variables: Option<Value>,
    ) -> Result<T, SubgraphError>
    where
        T: DeserializeOwned,
    {
        self.run_query_with_config(query, variables, SubgraphConfigOverride::default())
            .await
    }

    pub async fn run_query_with_config<T>(
        &self,
        query: &str,
        variables: Option<Value>,
        config_override: SubgraphConfigOverride,
    ) -> Result<T, SubgraphError>
    where
        T: DeserializeOwned,
    {
        let resolved_config = self.config_with_override(&config_override);
        let api = self.base_url_for(&resolved_config)?;
        let operation_name = extract_operation_name(query);
        let request = GraphQlRequest {
            query,
            variables: variables.as_ref(),
            operation_name: operation_name.as_deref(),
        };

        let response = self
            .client
            .post(&api)
            .json(&request)
            .send()
            .await
            .map_err(|error| SubgraphError::Transport {
                details: error.to_string(),
            })?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| SubgraphError::Transport {
                details: error.to_string(),
            })?;

        if !status.is_success() {
            return Err(SubgraphError::Transport {
                details: format!("http status {status}: {body}"),
            });
        }

        let response: GraphQlResponse<T> =
            serde_json::from_str(&body).map_err(|error| SubgraphError::Serialization {
                details: error.to_string(),
            })?;

        if !response.errors.is_empty() {
            return Err(SubgraphError::QueryFailed {
                query: query.to_owned(),
                variables: format_variables(variables.as_ref()),
                api,
                inner_error: serde_json::to_string(&response.errors)
                    .unwrap_or_else(|error| format!("failed to serialize GraphQL errors: {error}")),
            });
        }

        response.data.ok_or_else(|| SubgraphError::QueryFailed {
            query: query.to_owned(),
            variables: format_variables(variables.as_ref()),
            api,
            inner_error: "response missing data".to_owned(),
        })
    }

    fn config_with_override(&self, config_override: &SubgraphConfigOverride) -> SubgraphConfig {
        let mut config = self.config.clone();

        if let Some(chain_id) = config_override.chain_id {
            config.chain_id = chain_id;
        }

        if let Some(base_urls) = &config_override.base_urls {
            config.base_urls = Some(base_urls.clone());
        }

        config
    }

    fn base_url_for(&self, config: &SubgraphConfig) -> Result<String, SubgraphError> {
        let base_urls = config.base_urls.as_ref().unwrap_or(&self.prod_config);
        base_urls.get(&config.chain_id).cloned().flatten().ok_or(
            SubgraphError::UnsupportedNetwork {
                chain_id: config.chain_id as u64,
            },
        )
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphQlRequest<'a> {
    query: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<&'a Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    operation_name: Option<&'a str>,
}

#[derive(Deserialize)]
struct GraphQlResponse<T> {
    data: Option<T>,
    #[serde(default)]
    errors: Vec<GraphQlError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct GraphQlError {
    message: String,
    #[serde(default)]
    locations: Vec<GraphQlErrorLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct GraphQlErrorLocation {
    line: u32,
    column: u32,
}

fn build_prod_config(api_key: &str) -> SubgraphApiBaseUrls {
    let base_url = format!("{SUBGRAPH_BASE_URL}{api_key}/subgraphs/id");
    BTreeMap::from([
        (
            SupportedChainId::Mainnet,
            Some(format!(
                "{base_url}/8mdwJG7YCSwqfxUbhCypZvoubeZcFVpCHb4zmHhvuKTD"
            )),
        ),
        (
            SupportedChainId::GnosisChain,
            Some(format!(
                "{base_url}/HTQcP2gLuAy235CMNE8ApN4cbzpLVjjNxtCAUfpzRubq"
            )),
        ),
        (
            SupportedChainId::ArbitrumOne,
            Some(format!(
                "{base_url}/CQ8g2uJCjdAkUSNkVbd9oqqRP2GALKu1jJCD3fyY5tdc"
            )),
        ),
        (
            SupportedChainId::Base,
            Some(format!(
                "{base_url}/EYfBtJDj2thuBCVhdpYDpzfsWzDg3qzpEsitqMouU4Rg"
            )),
        ),
        (
            SupportedChainId::Sepolia,
            Some(format!(
                "{base_url}/31isonmztVX9ejBneP6SaVDQwEtyKCGBb3RTafB9Uf2y"
            )),
        ),
        (SupportedChainId::Polygon, None),
        (SupportedChainId::Avalanche, None),
        (SupportedChainId::Bnb, None),
        (SupportedChainId::Linea, None),
        (SupportedChainId::Plasma, None),
        (SupportedChainId::Ink, None),
    ])
}

fn extract_operation_name(query: &str) -> Option<String> {
    let trimmed = query.trim_start();

    for prefix in ["query", "mutation", "subscription"] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            let rest = rest.trim_start();

            if rest.starts_with('{') {
                return None;
            }

            let operation_name: String = rest
                .chars()
                .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
                .collect();

            if operation_name.is_empty() {
                return None;
            }

            return Some(operation_name);
        }
    }

    None
}

fn format_variables(variables: Option<&Value>) -> String {
    variables
        .map(ToString::to_string)
        .unwrap_or_else(|| "undefined".to_owned())
}
