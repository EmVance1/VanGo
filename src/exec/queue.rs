use std::path::Path;
use std::num::NonZero;
use std::process::{Child, Output};


pub struct ProcQueue<'a> {
    buffer: Vec<Option<(&'a Path, Child)>>,
    count: usize,
}

const BACKOFF_TIME: u64 = 10;

impl<'a> ProcQueue<'a> {
    pub fn new() -> Self {
        let threads = std::thread::available_parallelism().unwrap_or(NonZero::new(1).unwrap()).get();
        let mut result = Self{
            buffer: Vec::with_capacity(threads),
            count: 0,
        };
        result.buffer.resize_with(threads, || None);
        result
    }

    pub fn push<F: Fn(&'a Path, Output)->bool>(&mut self, elem: (&'a Path, Child), on_finish: F) -> Result<(), ()> {
        loop {
            for (i, handle) in self.buffer.iter_mut().enumerate() {
                if handle.is_none() {
                    self.buffer[i] = Some(elem);
                    self.count += 1;
                    return Ok(());

                } else if handle.as_mut().is_some_and(|(_, proc)| proc.try_wait().unwrap().is_some()) {
                    let (src, proc) = std::mem::take(handle).unwrap();
                    self.buffer[i] = Some(elem);
                    if !on_finish(src, proc.wait_with_output().unwrap()) {
                        return Err(())
                    } else {
                        return Ok(())
                    }
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

    #[allow(unused)]
    pub fn flush_one<F: Fn(&'a Path, Output)->bool>(&mut self, on_finish: F) -> Result<(), ()> {
        loop {
            for handle in self.buffer.iter_mut() {
                if handle.as_mut().is_some_and(|(_, proc)| proc.try_wait().unwrap().is_some()) {
                    let (src, proc) = std::mem::take(handle).unwrap();
                    if !on_finish(src, proc.wait_with_output().unwrap()) {
                        return Err(())
                    } else {
                        return Ok(())
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(BACKOFF_TIME));
        }
    }

    pub fn flush_all<F: Fn(&'a Path, Output)->bool>(&mut self, on_finish: F) -> Result<usize, usize> {
        let mut finished = 0;
        let mut failure = false;

        for handle in self.buffer.iter_mut() {
            if let Some((src, proc)) = std::mem::take(handle) {
                if !on_finish(src, proc.wait_with_output().unwrap()) {
                    failure = true;
                }
                finished += 1;
            }
        }
        self.count = 0;

        if failure {
            Err(finished)
        } else {
            Ok(finished)
        }
    }
}

