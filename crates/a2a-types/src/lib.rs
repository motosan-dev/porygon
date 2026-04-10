pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/lf.a2a.v1.rs"));
    include!(concat!(env!("OUT_DIR"), "/lf.a2a.v1.serde.rs"));
}

pub use proto::*;

/// JSON-RPC 2.0 request wrapper (not part of the proto spec).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 response wrapper.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: serde_json::Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_state_roundtrip() {
        let task = Task {
            id: "task-1".into(),
            context_id: "ctx-1".into(),
            status: Some(TaskStatus {
                state: TaskState::Completed as i32,
                ..Default::default()
            }),
            ..Default::default()
        };

        let json = serde_json::to_string(&task).unwrap();
        let parsed: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "task-1");
        assert_eq!(
            parsed.status.as_ref().unwrap().state,
            TaskState::Completed as i32
        );

        // Verify enum serializes as proto name
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["status"]["state"], "TASK_STATE_COMPLETED");
    }

    #[test]
    fn role_roundtrip() {
        let msg = Message {
            message_id: "m-1".into(),
            role: Role::User as i32,
            parts: vec![Part {
                content: Some(part::Content::Text("hello".into())),
                ..Default::default()
            }],
            ..Default::default()
        };

        let json = serde_json::to_string(&msg).unwrap();
        let parsed: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.role, Role::User as i32);

        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["role"], "ROLE_USER");
    }

    #[test]
    fn part_text_roundtrip() {
        let part = Part {
            content: Some(part::Content::Text("hello world".into())),
            media_type: "text/plain".into(),
            ..Default::default()
        };

        let json = serde_json::to_string(&part).unwrap();
        let parsed: Part = serde_json::from_str(&json).unwrap();
        match parsed.content.unwrap() {
            part::Content::Text(t) => assert_eq!(t, "hello world"),
            other => panic!("expected Text, got {other:?}"),
        }
        assert_eq!(parsed.media_type, "text/plain");
    }

    #[test]
    fn part_data_roundtrip() {
        let part = Part {
            content: Some(part::Content::Data(pbjson_types::Value {
                kind: Some(pbjson_types::value::Kind::StructValue(
                    pbjson_types::Struct {
                        fields: [("key".into(), pbjson_types::Value {
                            kind: Some(pbjson_types::value::Kind::StringValue("val".into())),
                        })]
                        .into(),
                    },
                )),
            })),
            ..Default::default()
        };

        let json = serde_json::to_string(&part).unwrap();
        let parsed: Part = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed.content, Some(part::Content::Data(_))));
    }

    #[test]
    fn send_message_request_roundtrip() {
        let req = SendMessageRequest {
            message: Some(Message {
                message_id: "msg-1".into(),
                role: Role::User as i32,
                parts: vec![Part {
                    content: Some(part::Content::Text("analyze AAPL".into())),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            configuration: Some(SendMessageConfiguration {
                accepted_output_modes: vec!["text/plain".into()],
                history_length: Some(10),
                ..Default::default()
            }),
            ..Default::default()
        };

        let json = serde_json::to_string(&req).unwrap();
        let parsed: SendMessageRequest = serde_json::from_str(&json).unwrap();

        let msg = parsed.message.unwrap();
        assert_eq!(msg.message_id, "msg-1");
        assert_eq!(msg.parts.len(), 1);

        let cfg = parsed.configuration.unwrap();
        assert_eq!(cfg.accepted_output_modes, vec!["text/plain"]);
        assert_eq!(cfg.history_length, Some(10));
    }

    #[test]
    fn agent_card_roundtrip() {
        let card = AgentCard {
            name: "my-agent".into(),
            description: "A test agent".into(),
            version: "1.0.0".into(),
            default_input_modes: vec!["text/plain".into()],
            default_output_modes: vec!["text/plain".into(), "application/json".into()],
            skills: vec![AgentSkill {
                id: "analyze".into(),
                name: "Analyze".into(),
                description: "Analyzes stocks".into(),
                tags: vec!["finance".into()],
                ..Default::default()
            }],
            capabilities: Some(AgentCapabilities {
                streaming: Some(true),
                push_notifications: Some(false),
                ..Default::default()
            }),
            ..Default::default()
        };

        let json = serde_json::to_string(&card).unwrap();
        let parsed: AgentCard = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "my-agent");
        assert_eq!(parsed.skills.len(), 1);
        assert_eq!(parsed.capabilities.unwrap().streaming, Some(true));
    }

    #[test]
    fn stream_response_status_update_roundtrip() {
        let resp = StreamResponse {
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
        };

        let json = serde_json::to_string(&resp).unwrap();
        let parsed: StreamResponse = serde_json::from_str(&json).unwrap();
        match parsed.payload.unwrap() {
            stream_response::Payload::StatusUpdate(e) => {
                assert_eq!(e.task_id, "task-1");
                assert_eq!(e.status.unwrap().state, TaskState::Working as i32);
            }
            other => panic!("expected StatusUpdate, got {other:?}"),
        }
    }

    #[test]
    fn artifact_roundtrip() {
        let artifact = Artifact {
            artifact_id: "art-1".into(),
            name: "report".into(),
            description: "Analysis report".into(),
            parts: vec![Part {
                content: Some(part::Content::Text("# Report".into())),
                media_type: "text/markdown".into(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let json = serde_json::to_string(&artifact).unwrap();
        let parsed: Artifact = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.artifact_id, "art-1");
        assert_eq!(parsed.parts.len(), 1);
    }

    #[test]
    fn jsonrpc_response_success() {
        let resp = JsonRpcResponse::success(
            serde_json::json!(1),
            serde_json::json!({"status": "ok"}),
        );
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: JsonRpcResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.jsonrpc, "2.0");
        assert!(parsed.error.is_none());
        assert_eq!(parsed.result.unwrap()["status"], "ok");
    }

    #[test]
    fn jsonrpc_response_error() {
        let resp = JsonRpcResponse::error(serde_json::json!(1), -32601, "method not found");
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: JsonRpcResponse = serde_json::from_str(&json).unwrap();
        assert!(parsed.result.is_none());
        let err = parsed.error.unwrap();
        assert_eq!(err.code, -32601);
        assert_eq!(err.message, "method not found");
    }
}
