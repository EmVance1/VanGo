use std::num::NonZero;
use std::process::{Child, Output};


pub struct ProcQueue {
    buffer: Vec<Option<Child>>,
    count: usize,
}

const BACKOFF_TIME: u64 = 10;

impl ProcQueue {
    pub fn new() -> Self {
        // preallocate vector with #threads child process slots
        let threads = std::thread::available_parallelism().unwrap_or(NonZero::new(1).unwrap()).get();
        let mut result = Self{
            buffer: Vec::with_capacity(threads),
            count: 0,
        };
        result.buffer.resize_with(threads, || None);
        result
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn push(&mut self, elem: Child) -> Option<Output> {
        // 'hot' loop acceptable as queue is only polled 100x per second, loop not actually hot
        loop {
            for (i, handle) in self.buffer.iter_mut().enumerate() {
                // IF queue not full, spawn process, enqueue
                if handle.is_none() {
                    self.buffer[i] = Some(elem);
                    self.count += 1;
                    return None

                // IF subprocess finishes, enqueue new process, return output from completed
                } else if handle.as_mut().is_some_and(|p| p.try_wait().unwrap().is_some()) {
                    let proc = std::mem::take(handle).unwrap();
                    self.buffer[i] = Some(elem);
                    return Some(proc.wait_with_output().unwrap())
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(BACKOFF_TIME));
        }

        /*
        let mut finished = 0;
        let mut failure = false;

        while slot.is_none() {
            for (i, handle) in self.buffer.iter_mut().enumerate() {
                if handle.is_none() {
                    // SLOT IS FREE
                    slot = Some(i);

                } else if handle.as_mut().is_some_and(|(_, proc)| proc.try_wait().unwrap().is_some()) {
                    // SLOT IS FULL :: AND :: SLOT IS FINISHED :: ==> :: FLUSH SLOT
                    let (src, proc) = std::mem::take(handle).unwrap();
                    if !on_finish(src, proc.wait_with_output().unwrap()) {
                        failure = true;
                    }
                    finished += 1;
                    self.count -= 1;
                    slot = Some(i);
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        self.buffer[slot.unwrap()] = Some(elem);
        self.count += 1;

        if failure {
            Err(finished)
        } else {
            Ok(finished)
        }
        */
    }

    pub fn flush_one(&mut self) -> Output {
        // wait until any subprocess finishes and return output
        // 'hot' loop acceptable as queue is polled only 100x per second, not actually hot
        loop {
            for handle in &mut self.buffer {
                if handle.as_mut().is_some_and(|p| p.try_wait().unwrap().is_some()) {
                    self.count -= 1;
                    return std::mem::take(handle).unwrap().wait_with_output().unwrap()
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(BACKOFF_TIME));
        }
    }
}

