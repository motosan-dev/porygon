# ag-ui-motosan

AG-UI protocol adapter for `motosan-agent-loop`. Translates `AgentEvent` into `AgUiAction` for SSE streaming to frontends.

Part of the [porygon](https://github.com/motosan-dev/porygon) workspace.

## Usage

```toml
[dependencies]
ag-ui-motosan = "0.1"
```

```rust
use ag_ui_motosan::{translate, AgUiAction};

// In your streaming callback:
agent.run_streaming(llm, messages, |event| {
    let actions = translate(event);
    for action in actions {
        match action {
            AgUiAction::TextChunk(delta) => { /* emit SSE text event */ }
            AgUiAction::TextDone(text) => { /* emit SSE text message */ }
            AgUiAction::ToolStarted { name } => { /* emit tool call start */ }
            AgUiAction::ToolCompleted { name, result } => { /* emit tool result */ }
        }
    }
});
```

## License

MIT
