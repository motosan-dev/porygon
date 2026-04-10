use a2a_types::{
    AgentCard, CancelTaskRequest, GetTaskRequest, ListTasksRequest, ListTasksResponse,
    SendMessageRequest, SendMessageResponse, StreamResponse, SubscribeToTaskRequest, Task,
};
use std::pin::Pin;
use tokio_stream::Stream;

/// Error type for A2A handler operations.
#[derive(Debug, thiserror::Error)]
pub enum A2AError {
    #[error("task not found: {0}")]
    TaskNotFound(String),
    #[error("method not found: {0}")]
    MethodNotFound(String),
    #[error("invalid params: {0}")]
    InvalidParams(String),
    #[error("internal error: {0}")]
    Internal(String),
    #[error("unsupported operation: {0}")]
    Unsupported(String),
}

impl A2AError {
    /// JSON-RPC error code.
    pub fn code(&self) -> i32 {
        match self {
            A2AError::MethodNotFound(_) => -32601,
            A2AError::InvalidParams(_) => -32602,
            A2AError::Internal(_) => -32603,
            A2AError::TaskNotFound(_) => -32001,
            A2AError::Unsupported(_) => -32002,
        }
    }
}

/// Trait for handling A2A protocol operations.
///
/// Implement this trait to create an A2A-compatible agent server.
/// Default implementations return `Unsupported` for optional methods.
#[async_trait::async_trait]
pub trait A2AHandler: Send + Sync + 'static {
    /// Returns the agent card describing this agent's capabilities.
    fn agent_card(&self) -> AgentCard;

    /// Send a message and get a response (blocking until terminal/interrupted state).
    async fn send_message(&self, req: SendMessageRequest) -> Result<SendMessageResponse, A2AError>;

    /// Send a message and stream responses via SSE.
    async fn send_streaming_message(
        &self,
        req: SendMessageRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamResponse, A2AError>> + Send>>, A2AError> {
        let _ = req;
        Err(A2AError::Unsupported("streaming not supported".into()))
    }

    /// Get the current state of a task.
    async fn get_task(&self, req: GetTaskRequest) -> Result<Task, A2AError> {
        let _ = req;
        Err(A2AError::Unsupported("get_task not supported".into()))
    }

    /// List tasks matching filter criteria.
    async fn list_tasks(&self, req: ListTasksRequest) -> Result<ListTasksResponse, A2AError> {
        let _ = req;
        Err(A2AError::Unsupported("list_tasks not supported".into()))
    }

    /// Cancel a task in progress.
    async fn cancel_task(&self, req: CancelTaskRequest) -> Result<Task, A2AError> {
        let _ = req;
        Err(A2AError::Unsupported("cancel_task not supported".into()))
    }

    /// Subscribe to task updates (streaming).
    async fn subscribe_to_task(
        &self,
        req: SubscribeToTaskRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamResponse, A2AError>> + Send>>, A2AError>
    {
        let _ = req;
        Err(A2AError::Unsupported(
            "subscribe_to_task not supported".into(),
        ))
    }
}
