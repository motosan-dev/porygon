//! Integration tests: real HTTP server + real client, end-to-end.

use std::pin::Pin;
use std::sync::Arc;

use a2a_client::{A2AClient, ClientError};
use a2a_server::{A2AError, A2AHandler};
use a2a_types::*;
use futures_util::StreamExt;
use tokio_stream::Stream;

// ---------------------------------------------------------------------------
// Test handler
// ---------------------------------------------------------------------------

struct TestAgent;

#[async_trait::async_trait]
impl A2AHandler for TestAgent {
    fn agent_card(&self) -> AgentCard {
        AgentCard {
            name: "test-agent".into(),
            description: "Integration test agent".into(),
            version: "1.0.0".into(),
            default_input_modes: vec!["text/plain".into()],
            default_output_modes: vec!["text/plain".into()],
            skills: vec![AgentSkill {
                id: "echo".into(),
                name: "Echo".into(),
                description: "Echoes back the input".into(),
                tags: vec!["test".into()],
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    async fn send_message(
        &self,
        req: SendMessageRequest,
    ) -> Result<SendMessageResponse, A2AError> {
        let input_text = req
            .message
            .as_ref()
            .and_then(|m| m.parts.first())
            .and_then(|p| p.content.as_ref())
            .and_then(|c| match c {
                part::Content::Text(t) => Some(t.clone()),
                _ => None,
            })
            .unwrap_or_default();

        Ok(SendMessageResponse {
            payload: Some(send_message_response::Payload::Task(Task {
                id: "task-123".into(),
                context_id: "ctx-1".into(),
                status: Some(TaskStatus {
                    state: TaskState::Completed as i32,
                    message: Some(Message {
                        message_id: "resp-1".into(),
                        role: Role::Agent as i32,
                        parts: vec![Part {
                            content: Some(part::Content::Text(format!("echo: {input_text}"))),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            })),
        })
    }

    async fn send_streaming_message(
        &self,
        req: SendMessageRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamResponse, A2AError>> + Send>>, A2AError>
    {
        let input_text = req
            .message
            .as_ref()
            .and_then(|m| m.parts.first())
            .and_then(|p| p.content.as_ref())
            .and_then(|c| match c {
                part::Content::Text(t) => Some(t.clone()),
                _ => None,
            })
            .unwrap_or_default();

        let events = vec![
            // 1. Working status
            Ok(StreamResponse {
                payload: Some(stream_response::Payload::StatusUpdate(
                    TaskStatusUpdateEvent {
                        task_id: "task-456".into(),
                        context_id: "ctx-1".into(),
                        status: Some(TaskStatus {
                            state: TaskState::Working as i32,
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                )),
            }),
            // 2. Artifact with echo
            Ok(StreamResponse {
                payload: Some(stream_response::Payload::ArtifactUpdate(
                    TaskArtifactUpdateEvent {
                        task_id: "task-456".into(),
                        context_id: "ctx-1".into(),
                        artifact: Some(Artifact {
                            artifact_id: "art-1".into(),
                            name: "echo".into(),
                            parts: vec![Part {
                                content: Some(part::Content::Text(format!(
                                    "echo: {input_text}"
                                ))),
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        last_chunk: true,
                        ..Default::default()
                    },
                )),
            }),
            // 3. Completed status
            Ok(StreamResponse {
                payload: Some(stream_response::Payload::StatusUpdate(
                    TaskStatusUpdateEvent {
                        task_id: "task-456".into(),
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

    async fn get_task(&self, req: GetTaskRequest) -> Result<Task, A2AError> {
        if req.id == "task-123" {
            Ok(Task {
                id: "task-123".into(),
                context_id: "ctx-1".into(),
                status: Some(TaskStatus {
                    state: TaskState::Completed as i32,
                    ..Default::default()
                }),
                ..Default::default()
            })
        } else {
            Err(A2AError::TaskNotFound(req.id))
        }
    }

    async fn cancel_task(&self, req: CancelTaskRequest) -> Result<Task, A2AError> {
        Ok(Task {
            id: req.id,
            status: Some(TaskStatus {
                state: TaskState::Canceled as i32,
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}

// ---------------------------------------------------------------------------
// Helper: start server on random port, return base URL
// ---------------------------------------------------------------------------

async fn start_server() -> String {
    let handler = Arc::new(TestAgent);
    let app = a2a_server::a2a_router(handler);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    format!("http://{addr}")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_get_agent_card() {
    let url = start_server().await;
    let client = A2AClient::new(&url);

    let card = client.get_agent_card().await.unwrap();
    assert_eq!(card.name, "test-agent");
    assert_eq!(card.version, "1.0.0");
    assert_eq!(card.skills.len(), 1);
    assert_eq!(card.skills[0].id, "echo");
}

#[tokio::test]
async fn e2e_send_message() {
    let url = start_server().await;
    let client = A2AClient::new(&url);

    let req = SendMessageRequest {
        message: Some(Message {
            message_id: "msg-1".into(),
            role: Role::User as i32,
            parts: vec![Part {
                content: Some(part::Content::Text("hello world".into())),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    };

    let resp = client.send_message(req).await.unwrap();
    let task = match resp.payload.unwrap() {
        send_message_response::Payload::Task(t) => t,
        _ => panic!("expected Task payload"),
    };

    assert_eq!(task.id, "task-123");
    assert_eq!(
        task.status.as_ref().unwrap().state,
        TaskState::Completed as i32
    );

    // Check the response message echoes the input
    let resp_msg = task.status.as_ref().unwrap().message.as_ref().unwrap();
    let text = match resp_msg.parts[0].content.as_ref().unwrap() {
        part::Content::Text(t) => t.clone(),
        _ => panic!("expected text part"),
    };
    assert_eq!(text, "echo: hello world");
}

#[tokio::test]
async fn e2e_get_task() {
    let url = start_server().await;
    let client = A2AClient::new(&url);

    let task = client.get_task("task-123").await.unwrap();
    assert_eq!(task.id, "task-123");
    assert_eq!(
        task.status.as_ref().unwrap().state,
        TaskState::Completed as i32
    );
}

#[tokio::test]
async fn e2e_get_task_not_found() {
    let url = start_server().await;
    let client = A2AClient::new(&url);

    let err = client.get_task("nonexistent").await.unwrap_err();
    match err {
        ClientError::Rpc { code, message, .. } => {
            assert_eq!(code, -32001); // TaskNotFound
            assert!(message.contains("nonexistent"));
        }
        other => panic!("expected RPC error, got: {other}"),
    }
}

#[tokio::test]
async fn e2e_cancel_task() {
    let url = start_server().await;
    let client = A2AClient::new(&url);

    let task = client.cancel_task("task-999").await.unwrap();
    assert_eq!(task.id, "task-999");
    assert_eq!(
        task.status.as_ref().unwrap().state,
        TaskState::Canceled as i32
    );
}

#[tokio::test]
async fn e2e_unsupported_list_tasks() {
    let url = start_server().await;
    let client = A2AClient::new(&url);

    let err = client
        .list_tasks(ListTasksRequest::default())
        .await
        .unwrap_err();
    match err {
        ClientError::Rpc { code, .. } => {
            assert_eq!(code, -32002); // Unsupported
        }
        other => panic!("expected RPC error, got: {other}"),
    }
}

#[tokio::test]
async fn e2e_streaming_message() {
    let url = start_server().await;
    let client = A2AClient::new(&url);

    let req = SendMessageRequest {
        message: Some(Message {
            message_id: "msg-stream-1".into(),
            role: Role::User as i32,
            parts: vec![Part {
                content: Some(part::Content::Text("streaming test".into())),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    };

    let stream = client.send_streaming_message(req).await.unwrap();
    tokio::pin!(stream);

    let mut events = Vec::new();
    while let Some(item) = stream.next().await {
        events.push(item.unwrap());
    }

    assert_eq!(events.len(), 3, "expected 3 stream events");

    // First: Working status
    match events[0].payload.as_ref().unwrap() {
        stream_response::Payload::StatusUpdate(e) => {
            assert_eq!(e.task_id, "task-456");
            assert_eq!(
                e.status.as_ref().unwrap().state,
                TaskState::Working as i32
            );
        }
        other => panic!("expected StatusUpdate, got: {other:?}"),
    }

    // Second: Artifact
    match events[1].payload.as_ref().unwrap() {
        stream_response::Payload::ArtifactUpdate(e) => {
            assert_eq!(e.task_id, "task-456");
            assert!(e.last_chunk);
            let art = e.artifact.as_ref().unwrap();
            let text = match art.parts[0].content.as_ref().unwrap() {
                part::Content::Text(t) => t.clone(),
                _ => panic!("expected text"),
            };
            assert_eq!(text, "echo: streaming test");
        }
        other => panic!("expected ArtifactUpdate, got: {other:?}"),
    }

    // Third: Completed status
    match events[2].payload.as_ref().unwrap() {
        stream_response::Payload::StatusUpdate(e) => {
            assert_eq!(
                e.status.as_ref().unwrap().state,
                TaskState::Completed as i32
            );
        }
        other => panic!("expected StatusUpdate, got: {other:?}"),
    }
}
