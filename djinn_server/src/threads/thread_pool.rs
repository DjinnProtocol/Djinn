use std::thread;

use async_std::{net::TcpStream, task::block_on, channel::{Sender, unbounded}};

use super::ThreadWorker;

pub struct ThreadPool {
    size: usize,
    senders: Vec<Sender<TcpStream>>,
    round_robin_id: usize,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let mut senders = Vec::with_capacity(size);
        for id in 0..size {
            let (sender, receiver) = unbounded();
            thread::spawn(move || {
                let mut worker = ThreadWorker::new(id, receiver);
                block_on(worker.run());
            });

            senders.push(sender);
        }
        debug!("Created {} threads", size);

        ThreadPool { senders, size, round_robin_id: 0 }
    }

    pub async fn distribute_stream(&mut self, stream: TcpStream) {
        // Send the stream to the next thread
        self.senders[self.round_robin_id].send(stream).await.unwrap();
        debug!("Sent stream to thread {}", self.round_robin_id);
        // Update the round robin id
        self.round_robin_id = (self.round_robin_id + 1) % self.size;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_pool() {
        let pool = ThreadPool::new(4);

        assert!(pool.size == 4);
        assert!(pool.round_robin_id == 0);
        assert!(pool.senders.len() == 4);
    }

}
