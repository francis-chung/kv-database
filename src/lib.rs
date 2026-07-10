use std::{
    sync::{Arc, Mutex, mpsc}, 
    thread
};

// sender is the sending end of mpsc
// wrapped in Option to facilitate dropping, which closes the channel
pub struct ThreadPool {
    workers: Vec<Worker>, 
    sender: Option<mpsc::Sender<Job>>
}

// Job must be a function, implement Send trait (its ownership can be transferred
// between threads), and doesn't borrow anything with a limited lifetime
// since each closure has a unique type, Boxing is necessary
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    // creates a new ThreadPool
    // panics if the size is 0
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        // receiver must be wrapped in Arc and Mutex in order to be sent to and
        // owned by multiple threads and exclusively mutated (received) in each
        let receiver = Arc::new(Mutex::new(receiver));

        // Arc clone creates a new pointer to allow for multiple ownership
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender: Some(sender) }
    }

    pub fn execute<F>(&self, f: F)
    where 
        F: FnOnce() + Send + 'static
    {
        // job needs to be boxed for type safety
        let job = Box::new(f);
        // as_ref borrows inner value of option
        if let Some(current_sender) = &self.sender.as_ref() {
            match current_sender.send(job) {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("Sending error: {e}");
                }
            }
        } else {
            eprintln!("Sender is None: channel disconnected or uninitialized.");
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        
        // drain used to bypass join's ownership of mutably borrowed
        // worker by removing worker after
        for worker in self.workers.drain(..) {
            println!("Shutting down worker {}", worker.id);
            match worker.thread.join() {
                Ok(_) => {}
                Err(e) => {
                    if let Some(msg) = e.downcast_ref::<&str>() {
                        eprintln!("Thread failed with &str: {msg}");
                    } else if let Some(msg) = e.downcast_ref::<String>() {
                        eprintln!("Thread failed with String: {msg}");
                    } else {
                        eprintln!("Thread failed with an unknown panic payload");
                    }
                }
            }
        }
    }
}

struct Worker {
    id: usize, 
    thread: thread::JoinHandle<()>
}

// worker function contains receiving end of mpsc
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                // lock is applied in the context of a mutex to return MutexGuard
                // accesses data anyway if lock is poisoned
                // recv blocks (pauses execution of thread)
                // lock is dropped once recv returns, so other threads can continue
                let message = receiver.lock()
                    .unwrap_or_else(|poisoned| {
                        eprintln!("Warning: Lock is poisoned. Recovering data");
                        poisoned.into_inner()
                    })
                    .recv();
                
                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");
                        job();
                    }
                    Err(_) => { // ThreadPool drop
                        println!("Worker {id} disconnected; shutting down.");
                        break;
                    }
                }
            }
        });
        
        Worker { id, thread }
    }
}