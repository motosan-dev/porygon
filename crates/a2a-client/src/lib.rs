use a2a_types::*;
use serde_json::Value;

/// Error type for A2A client operations.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("JSON-RPC error {code}: {message}")]
    Rpc {
        code: i32,
        message: String,
        data: Option<Value>,
    },
}

/// A2A protocol client for communicating with A2A-compatible agents.
pub struct A2AClient {
    http: reqwest::Client,
    base_url: String,
    next_id: std::sync::atomic::AtomicU64,
}

impl A2AClient {
    /// Create a new client pointing at the given agent URL.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            next_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Fetch the agent card from `/.well-known/agent.json`.
    pub async fn get_agent_card(&self) -> Result<AgentCard, ClientError> {
        let resp = self
            .http
            .get(format!("{}/.well-known/agent.json", self.base_url))
            .send()
            .await?
            .error_for_status()?;
        Ok(resp.json().await?)
    }

    /// Send a message to the agent (blocking call).
    pub async fn send_message(
        &self,
        req: SendMessageRequest,
    ) -> Result<SendMessageResponse, ClientError> {
        self.rpc_call("message/send", &req).await
    }

    /// Get a task by ID.
    pub async fn get_task(&self, id: impl Into<String>) -> Result<Task, ClientError> {
        let req = GetTaskRequest {
            id: id.into(),
            ..Default::default()
        };
        self.rpc_call("tasks/get", &req).await
    }

    /// List tasks with optional filters.
    pub async fn list_tasks(
        &self,
        req: ListTasksRequest,
    ) -> Result<ListTasksResponse, ClientError> {
        self.rpc_call("tasks/list", &req).await
    }

    /// Cancel a task.
    pub async fn cancel_task(&self, id: impl Into<String>) -> Result<Task, ClientError> {
        let req = CancelTaskRequest {
            id: id.into(),
            ..Default::default()
        };
        self.rpc_call("tasks/cancel", &req).await
    }

    /// Low-level JSON-RPC call.
    async fn rpc_call<P: serde::Serialize, R: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: &P,
    ) -> Result<R, ClientError> {
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let rpc_req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: serde_json::json!(id),
            method: method.into(),
            params: Some(serde_json::to_value(params)?),
        };

        let resp = self
            .http
            .post(&self.base_url)
            .json(&rpc_req)
            .send()
            .await?
            .error_for_status()?;

        let rpc_resp: JsonRpcResponse = resp.json().await?;

        if let Some(err) = rpc_resp.error {
            return Err(ClientError::Rpc {
                code: err.code,
                message: err.message,
                data: err.data,
            });
        }

        let result = rpc_resp
            .result
            .ok_or_else(|| ClientError::Rpc {
                code: -32603,
                message: "missing result".into(),
                data: None,
            })?;

        Ok(serde_json::from_value(result)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_strips_trailing_slash() {
        let client = A2AClient::new("http://localhost:3000/");
        assert_eq!(client.base_url, "http://localhost:3000");
    }
}
