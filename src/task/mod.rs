use core::{future::Future, pin::Pin};
use alloc::boxed::Box;
use core::task::{Context, Poll};
use core::sync::atomic::{AtomicU64, Ordering};

// Expose the child executor and driver modules cleanly to the compiler path
pub mod simple_executor;
pub mod keyboard;
pub mod executor;
pub mod shell;

/// A unique identifier tracking a specific asynchronous task state machine instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(pub(crate) u64); // Marked visible within the task module directory

impl TaskId {
    /// Generates an auto-incrementing atomic identifier variable thread-safely
    pub(crate) fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// A wrapper framework containing a heap-allocated, pinned asynchronous future state machine
pub struct Task {
    pub(crate) id: TaskId, // Exposed internally to executor systems
    pub(crate) future: Pin<Box<dyn Future<Output = ()> + 'static>>,
}

impl Task {
    /// Creates a fresh task wrapper wrapping a pinned, heap-allocated static future instance
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    /// Internal polling helper that advances the underlying future state structure
    pub(crate) fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}