use std::{
    num::NonZero,
    process::{Child, Output},
};

const BACKOFF_TIME: u64 = 10;

pub struct ProcessPool {
    buffer: Vec<Option<Child>>,
    count: usize,
}

impl ProcessPool {
    pub fn new() -> Self {
        // preallocate vector with #threads child process slots
        let threads = std::thread::available_parallelism().unwrap_or(NonZero::new(1).unwrap()).get();
        let mut result = Self {
            buffer: Vec::with_capacity(threads),
            count: 0,
        };
        result.buffer.resize_with(threads, || None);
        result
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Enqueue a task. If pool is full, blocks until a process exits
    pub fn enqueue_task(&mut self, elem: Child) -> Option<Output> {
        // 'hot' loop acceptable as queue is only polled 100x per second (loop not actually hot)
        loop {
            for (i, handle) in self.buffer.iter_mut().enumerate() {
                // If a slot is free, enqueue new process
                if handle.is_none() {
                    self.buffer[i] = Some(elem);
                    self.count += 1;
                    return None;

                // If a slot is finished, enqueue new process, return output from old process
                } else if handle.as_mut().is_some_and(|p| p.try_wait().unwrap().is_some()) {
                    let proc = std::mem::take(handle).unwrap();
                    self.buffer[i] = Some(elem);
                    return Some(proc.wait_with_output().unwrap());
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(BACKOFF_TIME));
        }
    }

    /// Block until any subprocess finishes and return output
    pub fn flush_one(&mut self) -> Output {
        // 'hot' loop acceptable as queue is only polled 100x per second (loop not actually hot)
        loop {
            for handle in &mut self.buffer {
                if handle.as_mut().is_some_and(|p| p.try_wait().unwrap().is_some()) {
                    self.count -= 1;
                    return std::mem::take(handle).unwrap().wait_with_output().unwrap();
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(BACKOFF_TIME));
        }
    }
}
