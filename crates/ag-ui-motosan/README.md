# porygon-ag-ui-motosan

AG-UI protocol adapter for [`motosan-agent-loop`](https://crates.io/crates/motosan-agent-loop). Translates `AgentEvent` into a flat `AgUiAction` stream for SSE streaming to frontends.

Part of the [porygon](https://github.com/motosan-dev/porygon) workspace.

Tracks `motosan-agent-loop` **0.13** — the two-layer `AgentEvent::Core(CoreEvent)` / `AgentEvent::Extension(ExtensionEvent)` shape.

## Usage

```toml
[dependencies]
porygon-ag-ui-motosan = "0.1"
```

The crate is imported as `ag_ui_motosan`:

```rust
use ag_ui_motosan::{translate, AgUiAction};
use motosan_agent_loop::AgentEvent;

// Inside your `on_event` callback for `Engine::run_streaming*`:
fn handle(event: AgentEvent) {
    for action in translate(event) {
        match action {
            AgUiAction::TextChunk(delta) => { /* emit SSE text event */ }
            AgUiAction::TextDone(text) => { /* emit SSE text message */ }
            AgUiAction::ToolStarted { name } => { /* emit tool call start */ }
            AgUiAction::ToolCompleted { name, result } => { /* emit tool result */ }
        }
    }
}
```

Events outside the mapped set (`CoreEvent::IterationStarted`, `Interrupted`, `Ops*`, `ExtensionFailed`, and every `Extension(_)` variant) yield an empty `Vec` and can be ignored by the caller.

## License

MIT
