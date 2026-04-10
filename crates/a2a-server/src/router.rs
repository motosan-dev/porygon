use std::convert::Infallible;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::State,
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use futures_util::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::Stream;

use a2a_types::{
    CancelTaskRequest, GetTaskRequest, JsonRpcRequest, JsonRpcResponse, ListTasksRequest,
    SendMessageRequest, StreamResponse, SubscribeToTaskRequest,
};

use crate::handler::{A2AError, A2AHandler};

/// Build an axum router for the A2A protocol.
///
/// Routes:
/// - `GET /.well-known/agent.json` — agent card discovery
/// - `POST /` — JSON-RPC dispatch for all A2A methods
pub fn a2a_router<H: A2AHandler>(handler: Arc<H>) -> Router {
    Router::new()
        .route("/.well-known/agent.json", get(agent_card_handler::<H>))
        .route("/", post(jsonrpc_handler::<H>))
        .with_state(handler)
}

async fn agent_card_handler<H: A2AHandler>(
    State(handler): State<Arc<H>>,
) -> impl IntoResponse {
    Json(handler.agent_card())
}

async fn jsonrpc_handler<H: A2AHandler>(
    State(handler): State<Arc<H>>,
    Json(mut req): Json<JsonRpcRequest>,
) -> axum::response::Response {
    let id = req.id.clone();

    match req.method.as_str() {
        "message/send" => {
            let params: SendMessageRequest = match parse_params(&mut req) {
                Ok(p) => p,
                Err(e) => return error_response(id, e),
            };
            match handler.send_message(params).await {
                Ok(result) => success_response(id, &result),
                Err(e) => error_response(id, e),
            }
        }
        "message/stream" => {
            let params: SendMessageRequest = match parse_params(&mut req) {
                Ok(p) => p,
                Err(e) => return error_response(id, e),
            };
            match handler.send_streaming_message(params).await {
                Ok(stream) => stream_to_sse(id, stream),
                Err(e) => error_response(id, e),
            }
        }
        "tasks/get" => {
            let params: GetTaskRequest = match parse_params(&mut req) {
                Ok(p) => p,
                Err(e) => return error_response(id, e),
            };
            match handler.get_task(params).await {
                Ok(task) => success_response(id, &task),
                Err(e) => error_response(id, e),
            }
        }
        "tasks/list" => {
            let params: ListTasksRequest = match parse_params(&mut req) {
                Ok(p) => p,
                Err(e) => return error_response(id, e),
            };
            match handler.list_tasks(params).await {
                Ok(result) => success_response(id, &result),
                Err(e) => error_response(id, e),
            }
        }
        "tasks/cancel" => {
            let params: CancelTaskRequest = match parse_params(&mut req) {
                Ok(p) => p,
                Err(e) => return error_response(id, e),
            };
            match handler.cancel_task(params).await {
                Ok(task) => success_response(id, &task),
                Err(e) => error_response(id, e),
            }
        }
        "tasks/subscribe" => {
            let params: SubscribeToTaskRequest = match parse_params(&mut req) {
                Ok(p) => p,
                Err(e) => return error_response(id, e),
            };
            match handler.subscribe_to_task(params).await {
                Ok(stream) => stream_to_sse(id, stream),
                Err(e) => error_response(id, e),
            }
        }
        method => error_response(id, A2AError::MethodNotFound(method.to_string())),
    }
}

/// Convert a stream of `StreamResponse` into an SSE response.
/// Each event is wrapped in a JSON-RPC response envelope.
fn stream_to_sse(
    id: serde_json::Value,
    stream: Pin<Box<dyn Stream<Item = Result<StreamResponse, A2AError>> + Send>>,
) -> axum::response::Response {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    tokio::spawn(async move {
        tokio::pin!(stream);
        while let Some(item) = stream.next().await {
            let json = match item {
                Ok(resp) => match serde_json::to_value(&resp) {
                    Ok(value) => {
                        let wrapped = JsonRpcResponse::success(id.clone(), value);
                        serde_json::to_string(&wrapped).unwrap()
                    }
                    Err(e) => {
                        tracing::warn!("failed to serialize stream response: {e}");
                        continue;
                    }
                },
                Err(e) => {
                    let wrapped = JsonRpcResponse::error(id.clone(), e.code(), e.to_string());
                    serde_json::to_string(&wrapped).unwrap()
                }
            };
            if tx.send(json).is_err() {
                break;
            }
        }
    });

    let sse_stream = UnboundedReceiverStream::new(rx)
        .map(|data| Ok::<_, Infallible>(Event::default().data(data)));

    Sse::new(sse_stream)
        .keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
        .into_response()
}

/// Parse JSON-RPC params into a typed request, taking ownership to avoid cloning.
fn parse_params<T: serde::de::DeserializeOwned>(
    req: &mut JsonRpcRequest,
) -> Result<T, A2AError> {
    let params = req
        .params
        .take()
        .ok_or_else(|| A2AError::InvalidParams("missing params".into()))?;
    serde_json::from_value(params).map_err(|e| A2AError::InvalidParams(e.to_string()))
}

fn success_response(
    id: serde_json::Value,
    result: &impl serde::Serialize,
) -> axum::response::Response {
    let value = serde_json::to_value(result).unwrap_or(serde_json::Value::Null);
    let resp = JsonRpcResponse::success(id, value);
    (StatusCode::OK, Json(resp)).into_response()
}

fn error_response(id: serde_json::Value, err: A2AError) -> axum::response::Response {
    let resp = JsonRpcResponse::error(id, err.code(), err.to_string());
    (StatusCode::OK, Json(resp)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use a2a_types::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    struct MockHandler;

    #[async_trait::async_trait]
    impl A2AHandler for MockHandler {
        fn agent_card(&self) -> AgentCard {
            AgentCard {
                name: "test-agent".into(),
                description: "A test agent".into(),
                version: "1.0.0".into(),
                ..Default::default()
            }
        }

        async fn send_message(
            &self,
            _req: SendMessageRequest,
        ) -> Result<SendMessageResponse, A2AError> {
            Ok(SendMessageResponse {
                payload: Some(send_message_response::Payload::Task(Task {
                    id: "task-1".into(),
                    status: Some(TaskStatus {
                        state: TaskState::Completed as i32,
                        ..Default::default()
                    }),
                    ..Default::default()
                })),
            })
        }

        async fn send_streaming_message(
            &self,
            _req: SendMessageRequest,
        ) -> Result<
            Pin<Box<dyn Stream<Item = Result<StreamResponse, A2AError>> + Send>>,
            A2AError,
        > {
            let events = vec![
                Ok(StreamResponse {
                    payload: Some(stream_response::Payload::StatusUpdate(
                        TaskStatusUpdateEvent {
                            task_id: "task-1".into(),
                            context_id: "ctx-1".into(),
                            status: Some(TaskStatus {
                                state: TaskState::Working as i32,
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                    )),
                }),
                Ok(StreamResponse {
                    payload: Some(stream_response::Payload::StatusUpdate(
                        TaskStatusUpdateEvent {
                            task_id: "task-1".into(),
                            context_id: "ctx-1".into(),
                            status: Some(TaskStatus {
                                state: TaskState::Completed as i32,
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                    )),
                }),
            ];
            Ok(Box::pin(tokio_stream::iter(events)))
        }
    }

    #[tokio::test]
    async fn agent_card_endpoint() {
        let app = a2a_router(Arc::new(MockHandler));

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/.well-known/agent.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let card: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(card["name"], "test-agent");
    }

    #[tokio::test]
    async fn jsonrpc_send_message() {
        let app = a2a_router(Arc::new(MockHandler));

        let rpc_req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: serde_json::json!(1),
            method: "message/send".into(),
            params: Some(serde_json::json!({
                "message": {
                    "messageId": "msg-1",
                    "role": "ROLE_USER",
                    "parts": [{"text": "hello"}]
                }
            })),
        };

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&rpc_req).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let rpc_resp: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
        assert!(rpc_resp.error.is_none());
        assert!(rpc_resp.result.is_some());
    }

    #[tokio::test]
    async fn jsonrpc_unknown_method() {
        let app = a2a_router(Arc::new(MockHandler));

        let rpc_req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: serde_json::json!(1),
            method: "nonexistent".into(),
            params: None,
        };

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&rpc_req).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let rpc_resp: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
        assert!(rpc_resp.error.is_some());
        assert_eq!(rpc_resp.error.unwrap().code, -32601);
    }

    #[tokio::test]
    async fn jsonrpc_unsupported_method_returns_error() {
        let app = a2a_router(Arc::new(MockHandler));

        let rpc_req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: serde_json::json!(1),
            method: "tasks/get".into(),
            params: Some(serde_json::json!({"id": "task-1"})),
        };

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&rpc_req).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let rpc_resp: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
        assert!(rpc_resp.error.is_some());
        assert_eq!(rpc_resp.error.unwrap().code, -32002); // Unsupported
    }

    #[tokio::test]
    async fn jsonrpc_stream_returns_sse_with_jsonrpc_envelope() {
        let app = a2a_router(Arc::new(MockHandler));

        let rpc_req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: serde_json::json!(42),
            method: "message/stream".into(),
            params: Some(serde_json::json!({
                "message": {
                    "messageId": "msg-1",
                    "role": "ROLE_USER",
                    "parts": [{"text": "hello"}]
                }
            })),
        };

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&rpc_req).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Parse SSE data lines
        let data_lines: Vec<&str> = body_str
            .lines()
            .filter(|l| l.starts_with("data: ") || l.starts_with("data:"))
            .collect();

        assert!(
            data_lines.len() >= 2,
            "expected at least 2 SSE data events, got {}: {body_str}",
            data_lines.len()
        );

        // Each SSE event should be a JSON-RPC response with id=42
        for line in &data_lines {
            let json_str = line.strip_prefix("data: ").or(line.strip_prefix("data:")).unwrap();
            let rpc_resp: JsonRpcResponse = serde_json::from_str(json_str)
                .unwrap_or_else(|e| panic!("failed to parse JSON-RPC response: {e}\nraw: {json_str}"));
            assert_eq!(rpc_resp.jsonrpc, "2.0");
            assert_eq!(rpc_resp.id, serde_json::json!(42));
            assert!(rpc_resp.result.is_some());
            assert!(rpc_resp.error.is_none());
        }
    }
}
