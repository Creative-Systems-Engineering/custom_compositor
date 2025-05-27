use tokio::sync::{mpsc, oneshot};
use futures::future::BoxFuture;
use std::future::Future;
use thiserror::Error;

/// Errors that can occur in async utilities
#[derive(Error, Debug)]
pub enum AsyncError {
    #[error("Task queue is disconnected")]
    Disconnected,
    #[error("Task execution failed")]
    TaskFailed,
    #[error("Channel error: {0}")]
    Channel(String),
}

/// A simple async task queue for managing compositor operations
pub struct TaskQueue {
    sender: mpsc::UnboundedSender<BoxFuture<'static, ()>>,
}

impl TaskQueue {
    pub fn new() -> (Self, TaskQueueHandle) {
        let (sender, receiver) = mpsc::unbounded_channel();
        let handle = TaskQueueHandle { receiver };
        let queue = Self { sender };
        (queue, handle)
    }
    
    /// Spawn a task on the queue
    pub fn spawn<F>(&self, future: F) -> Result<(), AsyncError>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.sender.send(Box::pin(future))
            .map_err(|_| AsyncError::Disconnected)
    }
    
    /// Spawn a task and wait for its completion
    pub async fn spawn_and_wait<F, T>(&self, future: F) -> Result<T, AsyncError>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        
        let task = async move {
            let result = future.await;
            let _ = tx.send(result);
        };
        
        self.spawn(task)?;
        
        rx.await.map_err(|_| AsyncError::TaskFailed)
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new().0
    }
}

/// Handle for processing tasks from the queue
pub struct TaskQueueHandle {
    receiver: mpsc::UnboundedReceiver<BoxFuture<'static, ()>>,
}

impl TaskQueueHandle {
    /// Process tasks from the queue
    pub async fn run(&mut self) {
        while let Some(task) = self.receiver.recv().await {
            task.await;
        }
    }
    
    /// Try to process one task if available
    pub fn try_process_one(&mut self) -> Option<BoxFuture<'static, ()>> {
        self.receiver.try_recv().ok()
    }
}

/// Utility for timeout operations
pub async fn timeout<F, T>(duration: std::time::Duration, future: F) -> Result<T, tokio::time::error::Elapsed>
where
    F: Future<Output = T>,
{
    tokio::time::timeout(duration, future).await
}

/// Utility for interval-based operations
pub fn create_interval(duration: std::time::Duration) -> tokio::time::Interval {
    tokio::time::interval(duration)
}
