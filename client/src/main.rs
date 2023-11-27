use std::net::{TcpStream};
use std::io::{BufRead, BufReader, stdin, Write};
use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

struct Client {
    stream: TcpStream,
}

impl Client {
    fn new<T: AsRef<str>>(url: T) -> Client {
        let stream = TcpStream::connect(url.as_ref()).unwrap();
        Client { stream }
    }

    fn fork(&self) -> Client {
        let stream = self.stream.try_clone().unwrap();
        Client { stream }
    }
}

fn main() {
    let run_flag = Arc::new(AtomicBool::new(true));

    loop {
        handle_client(run_flag.clone());
    }
}

fn handle_client(run_flag: Arc<AtomicBool>) {
    let client = Client::new("127.0.0.1:8080");
    let client_fork = client.fork();
    let run_clone = run_flag.clone();

    let handle = thread::spawn(move || {
        handle_response(client_fork, run_clone);
    });

    while run_flag.load(Ordering::SeqCst) {
        handle_stdin(&client);
    }

    handle.join().unwrap();
    println!("Reconnecting...");
    run_flag.store(true, Ordering::SeqCst);
}

fn handle_stdin(client: &Client) {
    let mut msg = String::new();
    stdin().read_line(&mut msg).unwrap();
    let mut stream = &client.stream;
    stream.write_all(msg.as_bytes()).unwrap();
}

fn handle_response(client: Client, run_flag: Arc<AtomicBool>) {
    let stream = client.stream;
    let mut buf_reader = BufReader::new(stream.try_clone().unwrap());
    loop {
        let mut line = String::new();
        if let Ok(_) = buf_reader.read_line(&mut line) {
            if line == "0000\n" {
                println!("finish message");
                break;
            }
            if !line.is_empty() {
                println!("Received: {}", line);
            }
        } else {
            println!("Disconnected from server");
            break;
        }
    }
    println!("over");
    run_flag.store(false, Ordering::SeqCst);
}
