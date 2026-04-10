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
    match event {
        motosan_agent_loop::AgentEvent::TextChunk(delta) => {
            vec![AgUiAction::TextChunk(delta)]
        }
        motosan_agent_loop::AgentEvent::TextDone(text) => {
            vec![AgUiAction::TextDone(text)]
        }
        motosan_agent_loop::AgentEvent::ToolStarted { name } => {
            vec![AgUiAction::ToolStarted { name }]
        }
        motosan_agent_loop::AgentEvent::ToolCompleted { name, result } => {
            let result_text = result.as_text().unwrap_or("").to_string();
            vec![AgUiAction::ToolCompleted {
                name,
                result: result_text,
            }]
        }
        // IterationStarted, Interrupted, AskUser, etc. — no ag-ui mapping
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use motosan_agent_tool::ToolResult;

    #[test]
    fn text_chunk_translates() {
        let actions = translate(motosan_agent_loop::AgentEvent::TextChunk("hello".into()));
        assert_eq!(actions.len(), 1);
        assert!(matches!(&actions[0], AgUiAction::TextChunk(s) if s == "hello"));
    }

    #[test]
    fn tool_completed_extracts_text() {
        let actions = translate(motosan_agent_loop::AgentEvent::ToolCompleted {
            name: "analyze".into(),
            result: ToolResult::text("result data"),
        });
        assert_eq!(actions.len(), 1);
        assert!(
            matches!(&actions[0], AgUiAction::ToolCompleted { name, result } if name == "analyze" && result == "result data")
        );
    }

    #[test]
    fn iteration_started_ignored() {
        let actions = translate(motosan_agent_loop::AgentEvent::IterationStarted(1));
        assert!(actions.is_empty());
    }
}
