use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::JoinHandle,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
        let mut num = 1;
        let name = format!("worker thread id: {}", id);

        let thread = Some(
            std::thread::Builder::new()
                .name(name.clone())
                .spawn(move || loop {
                    let msg = receiver.lock().expect("Failed to acquire the lock").recv();

                    match msg {
                        Ok(job) => {
                            println!("{} is receiving job number {}", name, num);
                            job();
                            num += 1;
                        }
                        Err(err) => {
                            eprintln!("{}. Shutting down {}", err, name);
                            break;
                        }
                    }
                })
                .unwrap(),
        );

        Self { id, thread }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // dropping sender first
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker id: {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let (sender, receiver) = channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            let worker = Worker::new(id, Arc::clone(&receiver));
            workers.push(worker);
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap()
    }
}
