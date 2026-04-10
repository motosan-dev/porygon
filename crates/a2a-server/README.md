# a2a-server

A2A v1.0 server for Rust. Provides an `A2AHandler` trait and an axum-based JSON-RPC router with SSE streaming support.

Part of the [porygon](https://github.com/motosan-dev/porygon) workspace.

## Usage

```toml
[dependencies]
a2a-server = "0.1"
a2a-types = "0.1"
```

Implement the `A2AHandler` trait:

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
            description: "My agent".into(),
            version: "1.0.0".into(),
            default_input_modes: vec!["text/plain".into()],
            default_output_modes: vec!["text/plain".into()],
            skills: vec![],
            ..Default::default()
        }
    }

    async fn send_message(&self, req: SendMessageRequest) -> Result<SendMessageResponse, A2AError> {
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

## Routes

- `GET /.well-known/agent.json` -- agent card discovery
- `POST /` -- JSON-RPC 2.0 dispatch for all A2A methods

## Supported Methods

All 11 A2A v1.0 methods. Only `send_message` is required; all others have default implementations that return `UnsupportedOperation`.

## License

MIT
