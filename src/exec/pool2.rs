use crate::Error;
use std::{
    io,
    process::{Child, Output},
    sync::mpsc::{self, Receiver, Sender},
};

pub struct ProcessPool {
    tx: Sender<io::Result<Output>>,
    rx: Receiver<io::Result<Output>>,
    capacity: usize,
    count: usize,
}

impl ProcessPool {
    pub fn new() -> Self {
        let capacity = std::thread::available_parallelism().map_or(1, |n| n.get());
        let (tx, rx) = mpsc::channel();
        Self { tx, rx, capacity, count: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Enqueue a task. If pool is full, blocks until a process exits and returns that process's output.
    pub fn enqueue_task(&mut self, mut cmd: std::process::Command, err: Error) -> Result<Option<Output>, Error> {
        let finished = if self.count >= self.capacity {
            Some(self.flush_one())
        } else {
            None
        };
        let child = cmd.spawn().map_err(|_| err)?;
        self.spawn_waiter(child);
        Ok(finished)
    }

    /// Block until any subprocess finishes and return output
    pub fn flush_one(&mut self) -> Output {
        assert!(self.count > 0, "flush_one called on empty pool");
        let output = self
            .rx
            .recv()
            .expect("pool channel closed")
            .expect("failed to wait on child process");
        self.count -= 1;
        output
    }

    fn spawn_waiter(&mut self, child: Child) {
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(child.wait_with_output());
        });
        self.count += 1;
    }
}

impl Default for ProcessPool {
    fn default() -> Self {
        Self::new()
    }
}

