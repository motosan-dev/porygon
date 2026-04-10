# Porygon

Agent protocol implementations in Rust. Covers [AG-UI](https://github.com/ag-ui-protocol/ag-ui) and [A2A](https://github.com/a2aproject/A2A) (v1.0).

## Crates

| Crate | Description |
|-------|-------------|
| [`a2a-types`](crates/a2a-types) | A2A v1.0 protocol types, auto-generated from the official protobuf via `prost` + `pbjson` |
| [`a2a-server`](crates/a2a-server) | A2A server with `A2AHandler` trait and axum JSON-RPC router |
| [`a2a-client`](crates/a2a-client) | A2A HTTP client with blocking and SSE streaming support |
| [`ag-ui-motosan`](crates/ag-ui-motosan) | AG-UI adapter for `motosan-agent-loop` |

## Quick Start

### A2A Server

Implement the `A2AHandler` trait and mount the router:

```rust
use a2a_server::{A2AHandler, A2AError, a2a_router};
use a2a_types::*;
use std::sync::Arc;

struct MyAgent;

#[async_trait::async_trait]
impl A2AHandler for MyAgent {
    fn agent_card(&self) -> AgentCard {
        AgentCard {
            name: "my-agent".into(),
            description: "My A2A agent".into(),
            version: "1.0.0".into(),
            default_input_modes: vec!["text/plain".into()],
            default_output_modes: vec!["text/plain".into()],
            skills: vec![AgentSkill {
                id: "chat".into(),
                name: "Chat".into(),
                description: "General chat".into(),
                tags: vec!["chat".into()],
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    async fn send_message(
        &self,
        req: SendMessageRequest,
    ) -> Result<SendMessageResponse, A2AError> {
        // Your agent logic here
        todo!()
    }
}

#[tokio::main]
async fn main() {
    let app = a2a_router(Arc::new(MyAgent));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### A2A Client

```rust
use a2a_client::A2AClient;
use a2a_types::*;

let client = A2AClient::new("http://localhost:3000");

// Discover agent
let card = client.get_agent_card().await?;

// Send a message
let req = SendMessageRequest {
    message: Some(Message {
        message_id: "msg-1".into(),
        role: Role::User as i32,
        parts: vec![Part {
            content: Some(part::Content::Text("Hello".into())),
            ..Default::default()
        }],
        ..Default::default()
    }),
    ..Default::default()
};
let resp = client.send_message(req).await?;
```

### A2A Client with Auth

```rust
let client = A2AClient::with_bearer_token("http://localhost:3000", "my-token");
```

## A2A Protocol Coverage

All 11 JSON-RPC methods from A2A v1.0 spec:

| Method | Server | Client |
|--------|--------|--------|
| `message/send` | handler trait | `send_message()` |
| `message/stream` | handler trait + SSE | `send_streaming_message()` |
| `tasks/get` | handler trait | `get_task()` |
| `tasks/list` | handler trait | `list_tasks()` |
| `tasks/cancel` | handler trait | `cancel_task()` |
| `tasks/subscribe` | handler trait + SSE | `subscribe_to_task()` |
| `tasks/pushNotificationConfig/set` | handler trait | - |
| `tasks/pushNotificationConfig/get` | handler trait | - |
| `tasks/pushNotificationConfig/list` | handler trait | - |
| `tasks/pushNotificationConfig/delete` | handler trait | - |
| `agent/extendedCard` | handler trait | - |

Error codes follow A2A v1.0 spec (-32001 through -32009).

## Updating A2A Types

Types are auto-generated from the official `a2a.proto`. To update:

```bash
# Copy latest proto from upstream
curl -sL https://raw.githubusercontent.com/a2aproject/A2A/main/specification/a2a.proto \
  | sed '/^import "google\/api/d' \
  | sed '/option (google\.api\.http)/,/};/d' \
  | sed '/option (google\.api\.method_signature)/d' \
  | sed 's/ \[(google\.api\.field_behavior) = [A-Z_]*\]//g' \
  | sed '/^option csharp_namespace/d;/^option go_package/d;/^option java/d' \
  > crates/a2a-types/proto/a2a.proto

# Remove the service block manually (we use JSON-RPC, not gRPC)
# Then rebuild
cargo build -p a2a-types
```

## License

MIT
