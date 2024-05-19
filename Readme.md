# Task Scheduler

This Rust library provides a simple task scheduler that allows you to schedule tasks to be executed after a certain delay. The tasks are executed in the order of their scheduled times.

## Features

- Schedule tasks with a delay
- Tasks are executed in the order they are scheduled
- Uses a background thread to manage task execution
- Thread-safe

## Usage

### Adding the dependency

Add this to your `Cargo.toml`:

```toml
[dependencies]
task_scheduler = "0.1.0"
```

### Example

Here's an example of how to use the `TaskScheduler`:

```rust
use task_scheduler::TaskScheduler;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn main() {
    let mut scheduler = TaskScheduler::new();
    scheduler.start();

    let counter = Arc::new(AtomicUsize::new(0));

    let counter_clone = Arc::clone(&counter);
    scheduler.schedule_task(
        Duration::from_secs(1),
        Box::new(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }),
    );

    let counter_clone = Arc::clone(&counter);
    scheduler.schedule_task(
        Duration::from_secs(0),
        Box::new(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }),
    );

    let counter_clone = Arc::clone(&counter);
    scheduler.schedule_task(
        Duration::from_secs(1),
        Box::new(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }),
    );

    // Wait for tasks to execute
    std::thread::sleep(Duration::from_secs(3));
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}
```

### Running Tests

To run the tests, use the following command:

```sh
cargo test
```

## Implementation Details

### Task Type

The `Task` type is defined as a boxed closure that takes no arguments and returns nothing:

```rust
type Task = Box<dyn FnOnce() + Send + 'static>;
```

### Scheduled Task Struct

The `ScheduledTask` struct contains the execution time and the task itself:

```rust
struct ScheduledTask {
    execute_at: Instant,
    task: Task,
}
```

### Task Scheduler

The `TaskScheduler` struct manages a heap of `ScheduledTask` instances and a condition variable to coordinate task execution:

```rust
pub struct TaskScheduler {
    tasks: Arc<(Mutex<BinaryHeap<ScheduledTask>>, Condvar)>,
}
```

### Methods

- `new()`: Creates a new `TaskScheduler`.
- `schedule_task()`: Schedules a task to be executed after a given delay.
- `start()`: Starts the background thread that executes the scheduled tasks.
