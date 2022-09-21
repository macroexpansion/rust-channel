use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

pub struct Inner<T> {
    queue: VecDeque<T>,
    num_sender: usize,
}

pub struct Shared<T> {
    inner: Mutex<Inner<T>>,
    is_available: Condvar,
}

pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Sender<T> {
    pub fn send(&mut self, message: T) {
        let mut inner = self.shared.inner.lock().unwrap(); // get the lock
        inner.queue.push_back(message);
        drop(inner); // drop the lock
        self.shared.is_available.notify_one();
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        let mut inner = self.shared.inner.lock().unwrap(); // get the lock
        inner.num_sender += 1;
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        // println!("drop");
        let mut inner = self.shared.inner.lock().unwrap(); // get the lock
        inner.num_sender -= 1;
        let is_last = inner.num_sender == 0;
        drop(inner);
        if is_last {
            self.shared.is_available.notify_one(); // notify one if we were the last
        }
    }
}

pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
    buffer: VecDeque<T>,
}

impl<T> Receiver<T> {
    pub fn recv(&mut self) -> Option<T> {
        if let Some(value) = self.buffer.pop_front() {
            return Some(value);
        }

        let mut inner = self.shared.inner.lock().unwrap();
        loop {
            match inner.queue.pop_front() {
                Some(value) => {
                    if !inner.queue.is_empty() {
                        std::mem::swap(&mut inner.queue, &mut self.buffer);
                    }

                    return Some(value);
                },
                None if dbg!(inner.num_sender) == 0 => return None,
                None => {
                    inner = self.shared.is_available.wait(inner).unwrap();
                }
            }
        }
    }
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Inner {
        queue: VecDeque::new(),
        num_sender: 1,
    };
    let shared = Shared {
        inner: Mutex::new(inner),
        is_available: Condvar::new(),
    };
    let shared = Arc::new(shared);
    (
        Sender {
            shared: Arc::clone(&shared),
        },
        Receiver {
            shared: Arc::clone(&shared),
            buffer: VecDeque::new(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let (mut sender, mut receiver) = channel::<i32>();

        for i in 0..10 {
            sender.send(5i32);
        }

        for i in 0..10 {
            let data = receiver.recv();
            assert_eq!(data, Some(5i32));
        }
    }

    #[test]
    fn zero_sender() {
        let (mut sender, mut receiver) = channel::<i32>();

        drop(sender);

        let data = receiver.recv();
        assert_eq!(data, None);
    }
}
