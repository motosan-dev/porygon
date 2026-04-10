# a2a-client

A2A v1.0 HTTP client for Rust. Supports both blocking JSON-RPC calls and SSE streaming.

Part of the [porygon](https://github.com/motosan-dev/porygon) workspace.

## Usage

```toml
[dependencies]
a2a-client = "0.1"
a2a-types = "0.1"
```

```rust
use a2a_client::A2AClient;
use a2a_types::*;

let client = A2AClient::new("http://localhost:3000");

// Discover agent capabilities
let card = client.get_agent_card().await?;

// Send a message
let resp = client.send_message(SendMessageRequest {
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
}).await?;

// Stream responses
use futures_util::StreamExt;
let stream = client.send_streaming_message(req).await?;
tokio::pin!(stream);
while let Some(event) = stream.next().await {
    println!("{event:?}");
}
```

## Authentication

```rust
let client = A2AClient::with_bearer_token("http://localhost:3000", "my-token");
```

## License

MIT
