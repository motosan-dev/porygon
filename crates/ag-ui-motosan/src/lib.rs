//! Adapter for the `motosan-agent-loop` crate.
//!
//! Translates `motosan_agent_loop::AgentEvent` into high-level actions
//! that a session state machine can process.

/// High-level action from the motosan agent framework.
/// The consuming session translates each action into ag-ui events.
#[derive(Debug, Clone)]
pub enum AgUiAction {
    /// Streaming text chunk from LLM.
    TextChunk(String),
    /// Full accumulated text after streaming completes.
    TextDone(String),
    /// A tool execution started.
    ToolStarted { name: String },
    /// A tool execution completed with a result.
    ToolCompleted { name: String, result: String },
}

/// Translates a [`motosan_agent_loop::AgentEvent`] into zero or more
/// [`AgUiAction`]s.
pub fn translate(event: motosan_agent_loop::AgentEvent) -> Vec<AgUiAction> {
    use motosan_agent_loop::{AgentEvent, CoreEvent};
    match event {
        AgentEvent::Core(CoreEvent::TextChunk(delta)) => {
            vec![AgUiAction::TextChunk(delta)]
        }
        AgentEvent::Core(CoreEvent::TextDone(text)) => {
            vec![AgUiAction::TextDone(text)]
        }
        AgentEvent::Core(CoreEvent::ToolStarted { name }) => {
            vec![AgUiAction::ToolStarted { name }]
        }
        AgentEvent::Core(CoreEvent::ToolCompleted { name, result }) => {
            let result_text = result.as_text().unwrap_or("").to_string();
            vec![AgUiAction::ToolCompleted {
                name,
                result: result_text,
            }]
        }
        // Other Core variants (IterationStarted, Interrupted, Ops*, ExtensionFailed)
        // and all Extension events — no ag-ui mapping.
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use motosan_agent_loop::{AgentEvent, CoreEvent};
    use motosan_agent_tool::ToolResult;

    #[test]
    fn text_chunk_translates() {
        let actions = translate(AgentEvent::Core(CoreEvent::TextChunk("hello".into())));
        assert_eq!(actions.len(), 1);
        assert!(matches!(&actions[0], AgUiAction::TextChunk(s) if s == "hello"));
    }

    #[test]
    fn tool_completed_extracts_text() {
        let actions = translate(AgentEvent::Core(CoreEvent::ToolCompleted {
            name: "analyze".into(),
            result: ToolResult::text("result data"),
        }));
        assert_eq!(actions.len(), 1);
        assert!(
            matches!(&actions[0], AgUiAction::ToolCompleted { name, result } if name == "analyze" && result == "result data")
        );
    }

    #[test]
    fn iteration_started_ignored() {
        let actions = translate(AgentEvent::Core(CoreEvent::IterationStarted {
            iteration: 1,
        }));
        assert!(actions.is_empty());
    }
}
