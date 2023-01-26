use std::{
    env,
    error::Error,
    fs,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    num::ParseIntError,
    str::FromStr,
    sync::{mpsc, Arc, Mutex, WaitTimeoutResult},
    thread,
};

pub struct ThreadPool {
    threads: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

type Job = Box<dyn FnOnce() -> () + Send + 'static>;

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
        F: FnOnce() -> () + Send + 'static,
    {
        self.sender.send(Box::new(job)).unwrap(); // send the job to one of the workers
    }
}

impl Worker {
    fn new(id: usize, reciever: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let job = reciever // send through mutex, as it is moved. revieve a job from the mpsc challel which is a closure
                .lock()
                .unwrap()
                .recv()
                .unwrap();
            println!("thread {id} is working");
            job(); // and do the job
        });

        Worker { id, thread }
    }
}

impl Restourant {
    fn new<E>() -> Result<Self, E>
    where
        E: std::convert::From<std::io::Error>,
    {
        let processfile = fs::read_to_string("$HOME/.config/hktcptsd/processes")?;

        let mut vector: Vec<Waiter> = Vec::new();

        for lines in processfile.lines() {
            match Waiter::from_str(lines) {
                Ok(t) => vector.push(t),
                _ => continue,
            }
        }

        Ok(Restourant { staff: vector })
    }
}

pub fn connection_handler(mut stream: TcpStream) {
    let mut write_stream = match stream.try_clone() {
        Ok(t) => t,
        Err(t) => panic!("[-]ERROR: {t}"),
    };

    let buff_reader = BufReader::new(&mut stream);

    let mut vector: Vec<String> = Vec::new();

    let iter = buff_reader.lines().take(3);
    for string in iter {
        match string {
            Ok(t) => vector.push(t),
            Err(t) => {
                write_stream.write_all("404".as_bytes()).unwrap();
                eprintln!("[-]ERROR: {t}");
            }
        }
    }

    let pass = vector.get(0).unwrap().to_string();
    let id: usize = vector.get(1).unwrap().trim().parse().unwrap();
    let command = vector.get(2).unwrap().to_string();

    println!("pass: {}", pass);
    println!("id: {}", id);
    println!("str: {}", command);

    let request = Request::from(pass, id, command);

    request.process();
}

struct Request {
    pass: String,
    id: usize,
    command: String,
}

impl Request {
    fn from(pass: String, id: usize, command: String) -> Self {
        Request { pass, id, command }
    }

    fn process(self) -> PwResult {
        if self.checkpw() {
            println!("your password was correct");

            let waiter = Waiter::from(self.id, self.command);

            return PwResult::Sucess;
        }

        println!("your password was not correc");

        PwResult::Fail
    }

    fn checkpw(&self) -> bool {
        if self.pass.eq("p") {
            return true;
        }
        false
    }
}

enum PwResult {
    Sucess,
    Fail,
}

pub struct Waiter {
    id: usize,
    command: String,
}

impl PartialEq for Waiter {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub struct Restourant {
    staff: Vec<Waiter>,
}

impl Waiter {
    fn from(id: usize, command: String) -> Self {
        Waiter { id, command }
    }
}

pub fn get_addr_thread() -> (String, usize) {
    let adress = match env::var("ADRESS") {
        Ok(t) => t,
        Err(_) => String::from("127.0.0.1:6969"), // if no value is provided, then the default will be this
    };

    let threads: usize = match env::var("MAX_PROCESS") {
        Ok(t) => t.parse().unwrap_or(8),
        Err(_) => 8, // if the value is not provided default will be 8
    };

    (adress, threads)
}

impl FromStr for Waiter {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let passed: Vec<&str> = s
            .trim_matches(|p| p == '(' || p == ')')
            .split(" ")
            .collect();

        let id = passed.get(0).unwrap().parse::<usize>()?;
        let command = passed.get(1).unwrap().trim();

        let waiter = Waiter::from(id, command.to_string());

        Ok(waiter)
    }
}
