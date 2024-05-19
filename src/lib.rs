use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

type Task = Box<dyn FnOnce() + Send + 'static>;

struct ScheduledTask {
    execute_at: Instant,
    task: Task,
}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse the order to get the smallest element first
        other.execute_at.cmp(&self.execute_at)
    }
}

impl PartialOrd for ScheduledTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ScheduledTask {
    fn eq(&self, other: &Self) -> bool {
        self.execute_at == other.execute_at
    }
}

impl Eq for ScheduledTask {}

pub struct TaskScheduler {
    tasks: Arc<(Mutex<BinaryHeap<ScheduledTask>>, Condvar)>,
}

impl TaskScheduler {
    pub fn new() -> Self {
        TaskScheduler {
            tasks: Arc::new((Mutex::new(BinaryHeap::new()), Condvar::new())),
        }
    }

    pub fn schedule_task(&mut self, delay: Duration, task: Task) {
        let execute_at = Instant::now() + delay;
        let scheduled_task = ScheduledTask { execute_at, task };

        let (lock, cvar) = &*self.tasks;
        let mut tasks = lock.lock().unwrap();
        tasks.push(scheduled_task);

        cvar.notify_one();
    }

    pub fn start(&self) {
        let tasks = self.tasks.clone();
        thread::spawn(move || {
            let (lock, cvar) = &*tasks;
            loop {
                let mut tasks = lock.lock().unwrap();
                if let Some(scheduled_task) = tasks.peek() {
                    let now = Instant::now();
                    if scheduled_task.execute_at <= now {
                        let task = tasks.pop().unwrap();
                        (task.task)();
                    } else {
                        let wait_time = scheduled_task.execute_at - now;
                        tasks = cvar.wait_timeout(tasks, wait_time).unwrap().0;
                    }
                } else {
                    tasks = cvar.wait(tasks).unwrap();
                }
            }
        });
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_task_scheduler() {
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

        thread::sleep(Duration::from_secs(3));
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }
}
