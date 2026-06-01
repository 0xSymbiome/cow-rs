/// Extension seam for downstream GraphQL adapters.
///
/// The current subgraph client owns its typed query execution directly. Keep
/// this as an adapter contract for consumers or future transport unification.
pub trait GraphTransport {
    /// Error type returned by transport operations.
    type Error;

    /// Executes a GraphQL request against the supplied endpoint.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined transport error when the request fails.
    fn execute(
        &self,
        endpoint: &str,
        query: &str,
        variables_json: Option<&str>,
    ) -> Result<String, Self::Error>;
}
