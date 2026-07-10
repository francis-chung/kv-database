use std::{
    sync::{Arc, Mutex, mpsc}, 
    thread
};

// sender is the sending end of mpsc
pub struct ThreadPool {
    workers: Vec<Worker>, 
    sender: mpsc::Sender<Job>
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

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where 
        F: FnOnce() + Send + 'static
    {
        // job needs to be boxed for type safety
        let job = Box::new(f);
        self.sender.send(job).unwrap();
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
                // recv blocks (pauses execution of thread)
                // lock is dropped once recv returns, so other threads can continue
                let job = receiver.lock().unwrap().recv().unwrap();
                println!("Worker {id} got a job; executing.");
                job();
            }
        });
        
        Worker { id, thread }
    }
}