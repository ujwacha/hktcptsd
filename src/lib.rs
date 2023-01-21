use std::{thread::{JoinHandle, self}, sync::{mpsc, Arc, Mutex}, net::TcpStream, io::BufReader};


pub struct ThreadPool{
    threads: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

struct Worker{
    id: usize,
    thread: thread::JoinHandle<()>
}


type Job =  Box<dyn FnOnce() -> () + Send + 'static>;

impl ThreadPool {

    pub fn new(number: usize) -> Self {
	assert!(number > 0);

	let (sender, reciever) = mpsc::channel();

	let reciever = Arc::new(Mutex::new(reciever));
	
	let mut threads = Vec::with_capacity(number);
	
	
	for id in 0..number {
	    let reciever = Arc::clone(&reciever);

	    threads.push(Worker::new(id, reciever));
	    
	}

	ThreadPool { threads, sender }

    }

    pub fn execute<F>(&self, job: F)
    where
	F: FnOnce() -> () + Send + 'static
    {

	self.sender.send(Box::new(job)).unwrap(); // send the job to one of the workers

    }
    
}

impl Worker {
    fn new(id: usize, reciever: Arc< Mutex<mpsc::Receiver<Job>>>) -> Self {
	let thread = thread::spawn(move || loop {
	    let job = reciever // send through mutex, as it is moved. revieve a job from the mpsc challel which is a closure
		.lock()
		.unwrap()
		.recv()
		.unwrap();

	    job(); // and do the job
	});

	Worker { id, thread }
	
    }
}
