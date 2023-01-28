use std::{
    borrow::Borrow,
    env, fs,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    num::ParseIntError,
    process::Command,
    rc::Rc,
    str::FromStr,
    sync::{mpsc, Arc, Mutex},
    thread,
};

const PROCESS_FILE: &'static str = "/home/light/.config/hktcptsd/processes";
const DEFAULT_ADRESS: &'static str = "127.0.0.1:6969";
const DEFAULT_NO_OF_THREADS: usize = 8;
const DEFAULT_PASSWOED: &'static str = r"rootadmin";

pub fn print_help() {
    println!("This is the Help Page");
    println!("Environment variables:");
    println!("ADRESS: for your adress default: {}", DEFAULT_ADRESS);
    println!(
        "MAX_PROCESS: number of threads default {}",
        DEFAULT_NO_OF_THREADS
    );
    println!("edit {} to set your processes", PROCESS_FILE);
}

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
    fn new() -> Self {
        let processfile = match fs::read_to_string(PROCESS_FILE) {
            Ok(t) => t,
            Err(t) => panic!("[-]ERROR: {t}"),
        };

        let mut staff: Vec<Waiter> = Vec::new();

        for lines in processfile.lines() {
            match Waiter::from_str(lines) {
                Ok(t) => staff.push(t),
                _ => continue,
            }
        }

        Restourant { staff }
    }
}

pub fn connection_handler(mut stream: TcpStream) {
    let password = get_password();

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
                panic!();
            }
        }
    }

    let pass = vector.get(0).unwrap().to_string();
    let id: usize = vector.get(1).unwrap().parse().unwrap();
    let command = vector.get(2).unwrap().to_string();

    dbg!(&pass);
    dbg!(&id);
    dbg!(&command);
    println!("pass: {}", pass);
    println!("id: {}", id);
    println!("str: {}", command);

    let request = Request::from(pass, id, command);

    let restourent = Restourant::new();

    request.process(restourent, password);
}

fn get_password() -> String {
    match env::var("HKTCPTSD_PASS") {
        Ok(t) => t.trim().to_string(),
        Err(_) => String::from(DEFAULT_PASSWOED),
    }
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

    fn process(self, restourent: Restourant, password: String) -> PwResult {
        println!("DEBUG INCOMING");
        dbg!(&restourent);
        println!("DEBUG DONE");

        if self.checkpw(password) {
            println!("your password was correct");

            let waiter = Rc::new(Waiter::from(self.id, self.command));

            println!("Waiter from the client incoming");
            dbg!(&waiter);
            println!("DONE");

            for option in restourent.staff {
                let waiter = Rc::clone(&waiter);

                if &option == waiter.borrow() {
                    let first_argument = waiter.command.clone();

                    env::set_var("STRING_VALUE", &first_argument);

                    println!("command: {} {}", &option.command, &first_argument);

                    let _command_output = Command::new("sh")
                        .arg(option.command)
                        .output()
                        .expect("failled to execute command");
                }
            }

            return PwResult::Sucess;
        }

        println!("your password was not correc");

        PwResult::Fail
    }

    fn checkpw(&self, password: String) -> bool {
        if self.pass.eq(&password) {
            return true;
        }
        false
    }
}

enum PwResult {
    Sucess,
    Fail,
}

#[derive(Debug)]
pub struct Waiter {
    id: usize,
    command: String,
}

impl PartialEq for Waiter {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug)]
pub struct Restourant {
    staff: Vec<Waiter>,
}

impl Waiter {
    fn from(id: usize, command: String) -> Self {
        Waiter { id, command }
    }
}

pub fn get_addr_thread() -> (String, usize) {
    let adress = match env::var("HKTCPTSD_ADRESS") {
        Ok(t) => t,
        Err(_) => String::from(DEFAULT_ADRESS), // if no value is provided, then the default will be this
    };

    let threads: usize = match env::var("HKTCPTSD_MAX_PROCESS") {
        Ok(t) => t.parse().unwrap_or(DEFAULT_NO_OF_THREADS),
        Err(_) => DEFAULT_NO_OF_THREADS, // if the value is not provided default will be 8
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
