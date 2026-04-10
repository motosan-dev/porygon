use a2a_types::{
    AgentCard, CancelTaskRequest, GetExtendedAgentCardRequest, GetTaskRequest,
    GetTaskPushNotificationConfigRequest, ListTaskPushNotificationConfigsRequest,
    ListTaskPushNotificationConfigsResponse, ListTasksRequest, ListTasksResponse,
    SendMessageRequest, SendMessageResponse, StreamResponse, SubscribeToTaskRequest, Task,
    TaskPushNotificationConfig,
};
use std::pin::Pin;
use tokio_stream::Stream;

/// Error type for A2A handler operations.
///
/// Error codes follow the A2A v1.0 specification:
/// - Standard JSON-RPC codes: -32600..-32603
/// - A2A-specific codes: -32001..-32009
#[derive(Debug, thiserror::Error)]
pub enum A2AError {
    #[error("task not found: {0}")]
    TaskNotFound(String),
    #[error("task not cancelable: {0}")]
    TaskNotCancelable(String),
    #[error("push notifications not supported")]
    PushNotificationNotSupported,
    #[error("unsupported operation: {0}")]
    UnsupportedOperation(String),
    #[error("content type not supported: {0}")]
    ContentTypeNotSupported(String),
    #[error("invalid agent response: {0}")]
    InvalidAgentResponse(String),
    #[error("extended agent card not configured")]
    ExtendedAgentCardNotConfigured,
    #[error("extension support required: {0}")]
    ExtensionSupportRequired(String),
    #[error("version not supported: {0}")]
    VersionNotSupported(String),
    // Standard JSON-RPC errors
    #[error("method not found: {0}")]
    MethodNotFound(String),
    #[error("invalid params: {0}")]
    InvalidParams(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl A2AError {
    /// JSON-RPC error code per A2A v1.0 specification.
    pub fn code(&self) -> i32 {
        match self {
            // Standard JSON-RPC codes
            A2AError::MethodNotFound(_) => -32601,
            A2AError::InvalidParams(_) => -32602,
            A2AError::Internal(_) => -32603,
            // A2A-specific codes (spec section 10)
            A2AError::TaskNotFound(_) => -32001,
            A2AError::TaskNotCancelable(_) => -32002,
            A2AError::PushNotificationNotSupported => -32003,
            A2AError::UnsupportedOperation(_) => -32004,
            A2AError::ContentTypeNotSupported(_) => -32005,
            A2AError::InvalidAgentResponse(_) => -32006,
            A2AError::ExtendedAgentCardNotConfigured => -32007,
            A2AError::ExtensionSupportRequired(_) => -32008,
            A2AError::VersionNotSupported(_) => -32009,
        }
    }
}

/// Trait for handling A2A protocol operations.
///
/// Implement this trait to create an A2A-compatible agent server.
/// Default implementations return `UnsupportedOperation` for optional methods.
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
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamResponse, A2AError>> + Send>>, A2AError>
    {
        let _ = req;
        Err(A2AError::UnsupportedOperation(
            "streaming not supported".into(),
        ))
    }

    /// Get the current state of a task.
    async fn get_task(&self, req: GetTaskRequest) -> Result<Task, A2AError> {
        let _ = req;
        Err(A2AError::UnsupportedOperation(
            "get_task not supported".into(),
        ))
    }

    /// List tasks matching filter criteria.
    async fn list_tasks(&self, req: ListTasksRequest) -> Result<ListTasksResponse, A2AError> {
        let _ = req;
        Err(A2AError::UnsupportedOperation(
            "list_tasks not supported".into(),
        ))
    }

    /// Cancel a task in progress.
    async fn cancel_task(&self, req: CancelTaskRequest) -> Result<Task, A2AError> {
        let _ = req;
        Err(A2AError::UnsupportedOperation(
            "cancel_task not supported".into(),
        ))
    }

    /// Subscribe to task updates (streaming).
    async fn subscribe_to_task(
        &self,
        req: SubscribeToTaskRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamResponse, A2AError>> + Send>>, A2AError>
    {
        let _ = req;
        Err(A2AError::UnsupportedOperation(
            "subscribe_to_task not supported".into(),
        ))
    }

    // -- Push notification methods --

    /// Create a push notification config for a task.
    async fn create_push_notification_config(
        &self,
        req: TaskPushNotificationConfig,
    ) -> Result<TaskPushNotificationConfig, A2AError> {
        let _ = req;
        Err(A2AError::PushNotificationNotSupported)
    }

    /// Get a push notification config.
    async fn get_push_notification_config(
        &self,
        req: GetTaskPushNotificationConfigRequest,
    ) -> Result<TaskPushNotificationConfig, A2AError> {
        let _ = req;
        Err(A2AError::PushNotificationNotSupported)
    }

    /// List push notification configs for a task.
    async fn list_push_notification_configs(
        &self,
        req: ListTaskPushNotificationConfigsRequest,
    ) -> Result<ListTaskPushNotificationConfigsResponse, A2AError> {
        let _ = req;
        Err(A2AError::PushNotificationNotSupported)
    }

    /// Delete a push notification config.
    async fn delete_push_notification_config(
        &self,
        req: a2a_types::DeleteTaskPushNotificationConfigRequest,
    ) -> Result<(), A2AError> {
        let _ = req;
        Err(A2AError::PushNotificationNotSupported)
    }

    /// Get extended agent card (requires authentication).
    async fn get_extended_agent_card(
        &self,
        req: GetExtendedAgentCardRequest,
    ) -> Result<AgentCard, A2AError> {
        let _ = req;
        Err(A2AError::ExtendedAgentCardNotConfigured)
    }
}
