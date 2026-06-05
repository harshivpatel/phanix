use super::{Task, TaskId};
use alloc::{collections::BTreeMap, sync::Arc};
use core::task::Waker;
use core::task::{Context, Poll};
use alloc::task::Wake;
use crossbeam_queue::ArrayQueue;

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    /// Creates a new instance of the single-threaded asynchronous task executor
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            // Safe capacity allocation for processing up to 100 concurrent task events
            task_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }

    /// Spawns an asynchronous task by placing its future inside our tracking map
    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same ID already in tasks");
        }
        self.task_queue.push(task_id).expect("queue full");
    }

    /// Iterates through the lock-free array queue, polling ready tasks until empty
    fn run_ready_tasks(&mut self) {
        // Destructure self to satisfy the strict Rust structural borrow checker boundaries
        let Self {
            tasks,
            task_queue,
            waker_cache,
        } = self;

        while let Some(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // The task does not exist or has already finished execution
            };
            
            // Fetch the cached waker instance or initialize a fresh one for this task ID
            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
                
            let mut context = Context::from_waker(waker);
            
            // Poll the individual task state machine future
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // Task execution complete: clear it completely from our tracking maps
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {} // Still waiting for a hardware signal, keep it in the map
            }
        }
    }

    /// The runtime entry point loop that drives the task execution ecosystem indefinitely
    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle(); 
        }
    }

    /// Safely halts the CPU core when no tasks are ready, preventing power waste
    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts::{self, enable_and_hlt};

        // Disable interrupts to ensure our check is completely atomic
        interrupts::disable();
        if self.task_queue.is_empty() {
            // Atomically re-enables hardware interrupts and puts the CPU core into a low-power HLT state
            enable_and_hlt();
        } else {
            // Re-enable interrupts without halting since a task was queued in the interim
            interrupts::enable();
        }
    }
}

/// A structure wrapping the necessary elements to awake a task safely across execution contexts
struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    /// Factory constructor helper that instantiates a safe Waker primitive instance
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    /// Pushes the given task ID back onto our ready queue execution pipeline
    fn wake_task(&self) {
        // Use loose handling or an explicit check instead of panicking on full queue conditions during heavy I/O load
        let _ = self.task_queue.push(self.task_id);
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}