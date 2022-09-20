use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

pub struct Inner<T> {
    queue: Mutex<VecDeque<T>>,
}

pub struct Sender<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Sender<T> {
    pub fn send(&mut self, message: T) {
        let queue = self.inner.queue.lock().unwrap();
        queue.push_back(message);
    }
}

pub struct Receiver<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Receiver<T> {
    pub fn recv(&mut self) -> T {
        let queue = self.inner.queue.lock().unwrap();
        message = queue.pop_front();
    }
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Inner {
        queue: Mutex::default(),
    };
    let inner = Arc::new(inner);
    (
        Sender {
            inner: inner.clone(),
        },
        Receiver {
            inner: inner.clone(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let (sender, receiver) = channel::<i32>();
    }
}
