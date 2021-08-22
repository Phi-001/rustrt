use std::sync::mpsc;
use std::thread;

type Job<T> = Box<dyn Fn(T) + 'static + Send>;

pub struct ThreadPool<T: Send + 'static> {
    senders: Vec<mpsc::Sender<T>>,
    receiver: mpsc::Receiver<usize>,
    workers: Vec<Worker>,
    available_workers: Vec<usize>,
    command_que: Vec<T>,
    pub size: usize,
}

impl<T: Send + 'static> ThreadPool<T> {
    pub fn new<F>(size: usize, job: F) -> Self
    where
        F: Fn() -> Job<T>,
    {
        let mut senders = Vec::with_capacity(size);
        let mut workers = Vec::with_capacity(size);

        let (tx_finish, rx_finish) = mpsc::channel();

        for id in 0..size {
            let (tx, rx) = mpsc::channel();
            workers.push(Worker::new(rx, tx_finish.clone(), job(), id));
            senders.push(tx);
        }

        ThreadPool {
            senders,
            receiver: rx_finish,
            workers,
            available_workers: (0..size).collect(),
            command_que: vec![],
            size,
        }
    }

    pub fn push_que(&mut self, work: T) {
        self.command_que.push(work);
    }

    pub fn execute_que(&mut self) {
        while !self.command_que.is_empty() {
            let argument = self.command_que.pop().unwrap();
            let available = if !self.available_workers.is_empty() {
                self.available_workers.pop().unwrap()
            } else {
                self.receiver.recv().unwrap()
            };
            self.senders[available].send(argument).unwrap();
        }
    }
}

impl<T: Send + 'static> Drop for ThreadPool<T> {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new<T: Send + 'static>(
        receiver: mpsc::Receiver<T>,
        sender: mpsc::Sender<usize>,
        work: Job<T>,
        id: usize,
    ) -> Self {
        let thread = thread::spawn(move || {
            for arg in receiver {
                work(arg);
                sender.send(id).unwrap();
            }
        });
        Worker {
            thread: Some(thread),
        }
    }
}
