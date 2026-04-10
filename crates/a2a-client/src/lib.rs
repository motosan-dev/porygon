use a2a_types::*;
use futures_util::{Stream, StreamExt};
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
    #[error("stream error: {0}")]
    Stream(String),
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

    /// Create a new client with a bearer token for authentication.
    pub fn with_bearer_token(base_url: impl Into<String>, token: &str) -> Self {
        use reqwest::header;
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {token}"))
                .expect("invalid token"),
        );
        Self {
            http: reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .expect("failed to build HTTP client"),
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

    /// Send a message and stream responses via SSE.
    ///
    /// Returns a stream of `StreamResponse` items parsed from SSE events.
    /// Each SSE event is expected to be a JSON-RPC response wrapping a `StreamResponse`.
    pub async fn send_streaming_message(
        &self,
        req: SendMessageRequest,
    ) -> Result<impl Stream<Item = Result<StreamResponse, ClientError>>, ClientError> {
        self.rpc_stream("message/stream", &req).await
    }

    /// Subscribe to task updates via SSE.
    pub async fn subscribe_to_task(
        &self,
        task_id: impl Into<String>,
    ) -> Result<impl Stream<Item = Result<StreamResponse, ClientError>>, ClientError> {
        let req = SubscribeToTaskRequest {
            id: task_id.into(),
            ..Default::default()
        };
        self.rpc_stream("tasks/subscribe", &req).await
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

    fn next_id(&self) -> u64 {
        self.next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    /// JSON-RPC call that returns a single result.
    async fn rpc_call<P: serde::Serialize, R: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: &P,
    ) -> Result<R, ClientError> {
        let rpc_req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: serde_json::json!(self.next_id()),
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

        let result = rpc_resp.result.ok_or_else(|| ClientError::Rpc {
            code: -32603,
            message: "missing result".into(),
            data: None,
        })?;

        Ok(serde_json::from_value(result)?)
    }

    /// JSON-RPC call that returns an SSE stream.
    /// Each SSE `data:` line is a JSON-RPC response wrapping a `StreamResponse`.
    async fn rpc_stream(
        &self,
        method: &str,
        params: &impl serde::Serialize,
    ) -> Result<impl Stream<Item = Result<StreamResponse, ClientError>>, ClientError> {
        let rpc_req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: serde_json::json!(self.next_id()),
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

        // Read SSE stream line by line
        let byte_stream = resp.bytes_stream();

        Ok(parse_sse_stream(byte_stream))
    }
}

/// Parse an SSE byte stream into `StreamResponse` items.
/// Expects each SSE event to be a JSON-RPC response with a `StreamResponse` as the result.
fn parse_sse_stream(
    byte_stream: impl Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send + 'static,
) -> impl Stream<Item = Result<StreamResponse, ClientError>> {
    async_stream::stream! {
        let mut buffer = String::new();

        tokio::pin!(byte_stream);
        while let Some(chunk) = byte_stream.next().await {
            let chunk = match chunk {
                Ok(c) => c,
                Err(e) => {
                    yield Err(ClientError::Http(e));
                    break;
                }
            };

            buffer.push_str(&String::from_utf8_lossy(&chunk));

            // Process complete lines
            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].trim().to_string();
                buffer = buffer[pos + 1..].to_string();

                if line.is_empty() || line.starts_with(':') {
                    continue; // SSE comment or blank line
                }

                if let Some(data) = line.strip_prefix("data: ").or_else(|| line.strip_prefix("data:")) {
                    // Parse JSON-RPC response
                    match serde_json::from_str::<JsonRpcResponse>(data) {
                        Ok(rpc_resp) => {
                            if let Some(err) = rpc_resp.error {
                                yield Err(ClientError::Rpc {
                                    code: err.code,
                                    message: err.message,
                                    data: err.data,
                                });
                                break;
                            }
                            if let Some(result) = rpc_resp.result {
                                match serde_json::from_value::<StreamResponse>(result) {
                                    Ok(sr) => yield Ok(sr),
                                    Err(e) => yield Err(ClientError::Json(e)),
                                }
                            }
                        }
                        Err(e) => {
                            yield Err(ClientError::Stream(
                                format!("failed to parse SSE data as JSON-RPC: {e}"),
                            ));
                        }
                    }
                }
            }
        }
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

    #[test]
    fn client_with_bearer_token() {
        let client = A2AClient::with_bearer_token("http://localhost:3000", "my-token");
        assert_eq!(client.base_url, "http://localhost:3000");
    }
}
