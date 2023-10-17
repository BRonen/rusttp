use std::{
    thread,
    sync::{mpsc, Arc, Mutex},
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

fn handle_connection(thread_id: usize, mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let _http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let status_line = "HTTP/1.1 716 OK";

    let contents = format!("{{\"test\": \"hello world\", \"id\": \"{thread_id}\"}}\n");
    let length = contents.len();

    let response =
    format!("{status_line}\r\nContent-Type: application/json\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<Arc<Mutex<mpsc::Receiver<Job>>>>
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            while let Ok(job) = receiver.lock().unwrap().recv() {
                println!("Worker {id} got a job; executing.");

                job(id);
            }
        });

        Worker { id, thread }
    }
}

struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce(usize) + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        
        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        
        let receiver = Arc::new(Mutex::new(receiver));
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        
        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(usize) + Send + 'static,
    {
        let job = Box::new(f);
    
        self.sender.send(job).unwrap();
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
    
    let pool = ThreadPool::new(4);
    
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|id: usize| {
            handle_connection(id, stream);
        })
    }
}