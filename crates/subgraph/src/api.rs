use std::collections::BTreeMap;

use cow_sdk_core::{HttpClientPolicy, SupportedChainId};
use reqwest::Client;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use crate::{
    error::SubgraphError,
    queries::{LAST_DAYS_VOLUME_QUERY, LAST_HOURS_VOLUME_QUERY, TOTALS_QUERY},
    types::{
        LastDaysVolumeResponse, LastHoursVolumeResponse, SubgraphQueryRequest, Total,
        TotalsResponse,
    },
};

const SUBGRAPH_BASE_URL: &str = "https://gateway.thegraph.com/api/";

pub const API_NAME: &str = "CoW Protocol Subgraph";
pub const DEFAULT_SUBGRAPH_USER_AGENT: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubgraphTransportPolicy {
    client: HttpClientPolicy,
}

impl Default for SubgraphTransportPolicy {
    fn default() -> Self {
        Self {
            client: HttpClientPolicy::new(DEFAULT_SUBGRAPH_USER_AGENT)
                .expect("static subgraph user-agent must remain valid"),
        }
    }
}

impl SubgraphTransportPolicy {
    pub fn new(client: HttpClientPolicy) -> Self {
        Self { client }
    }

    pub fn client_policy(&self) -> &HttpClientPolicy {
        &self.client
    }

    pub fn with_client_policy(mut self, client: HttpClientPolicy) -> Self {
        self.client = client;
        self
    }
}

#[derive(Clone)]
pub struct SubgraphApi {
    client: Client,
    config: SubgraphConfig,
    prod_config: SubgraphApiBaseUrls,
    transport_policy: SubgraphTransportPolicy,
}

impl SubgraphApi {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_config(api_key, SubgraphConfig::default())
    }

    pub fn with_config(api_key: impl Into<String>, config: SubgraphConfig) -> Self {
        Self::with_config_and_transport_policy(api_key, config, SubgraphTransportPolicy::default())
    }

    pub fn with_config_and_transport_policy(
        api_key: impl Into<String>,
        config: SubgraphConfig,
        transport_policy: SubgraphTransportPolicy,
    ) -> Self {
        let api_key = api_key.into();

        Self {
            client: build_client(transport_policy.client_policy()),
            prod_config: build_prod_config(&api_key),
            config,
            transport_policy,
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

    pub fn transport_policy(&self) -> &SubgraphTransportPolicy {
        &self.transport_policy
    }

    pub fn client_policy(&self) -> &HttpClientPolicy {
        self.transport_policy.client_policy()
    }

    pub fn with_transport_policy(mut self, transport_policy: SubgraphTransportPolicy) -> Self {
        self.client = build_client(transport_policy.client_policy());
        self.transport_policy = transport_policy;
        self
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
            .run_query_with_config(
                SubgraphQueryRequest::new(TOTALS_QUERY).with_operation_name("Totals"),
                config_override,
            )
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
            SubgraphQueryRequest::new(LAST_DAYS_VOLUME_QUERY)
                .with_variables(json!({ "days": days }))
                .with_operation_name("LastDaysVolume"),
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
            SubgraphQueryRequest::new(LAST_HOURS_VOLUME_QUERY)
                .with_variables(json!({ "hours": hours }))
                .with_operation_name("LastHoursVolume"),
            config_override,
        )
        .await
    }

    pub async fn run_query<T, R>(&self, request: R) -> Result<T, SubgraphError>
    where
        T: DeserializeOwned,
        R: Into<SubgraphQueryRequest>,
    {
        self.run_query_with_config(request, SubgraphConfigOverride::default())
            .await
    }

    pub async fn run_query_with_config<T, R>(
        &self,
        request: R,
        config_override: SubgraphConfigOverride,
    ) -> Result<T, SubgraphError>
    where
        T: DeserializeOwned,
        R: Into<SubgraphQueryRequest>,
    {
        let request = request.into();
        let resolved_config = self.config_with_override(&config_override);
        let api = self.base_url_for(&resolved_config)?;
        let graphql_request = GraphQlRequest {
            query: request.document(),
            variables: request.variables(),
            operation_name: request.operation_name(),
        };

        let mut request_builder = self.client.post(&api).json(&graphql_request);

        if let Some(timeout) = self.client_policy().timeout() {
            request_builder = request_builder.timeout(timeout);
        }

        let response = request_builder
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
                query: request.document().to_owned(),
                variables: format_variables(request.variables()),
                api,
                inner_error: serde_json::to_string(&response.errors)
                    .unwrap_or_else(|error| format!("failed to serialize GraphQL errors: {error}")),
            });
        }

        response.data.ok_or_else(|| SubgraphError::QueryFailed {
            query: request.document().to_owned(),
            variables: format_variables(request.variables()),
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

fn format_variables(variables: Option<&Value>) -> String {
    variables
        .map(ToString::to_string)
        .unwrap_or_else(|| "undefined".to_owned())
}

fn build_client(policy: &HttpClientPolicy) -> Client {
    let builder = Client::builder().user_agent(policy.user_agent().to_owned());

    builder
        .build()
        .expect("validated subgraph client policy must remain buildable")
}
