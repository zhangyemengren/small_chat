use std::io::{stdin, BufRead, BufReader, Write};
use std::net::{TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex};
use std::thread;

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
const HEARTBEAT_INTERVAL: usize = 5;
static CLIENT: Mutex<Option<Client>> = Mutex::new(None);
static RUN_FLAG: AtomicBool = AtomicBool::new(true);
static MESSAGE_QUEUE: Mutex<Vec<String>> = Mutex::new(Vec::new());
static HEARTBEAT_COUNT: Mutex<usize> = Mutex::new(HEARTBEAT_INTERVAL);

fn main() {
    handle_client();
}

fn reconnect() {
    let mut client = CLIENT.lock().unwrap();
    *client = Some(Client::new("127.0.0.1:8080"));
    RUN_FLAG.store(true, Ordering::SeqCst);
    thread::spawn(move || {
        handle_response();
    });
    thread::spawn(move || {
        handle_heartbeat();
    });
    let mut msg_queue = MESSAGE_QUEUE.lock().unwrap();
    if msg_queue.len() > 0 {
        let mut stream = &client.as_ref().unwrap().stream;
        for msg in msg_queue.iter() {
            stream.write_all(msg.as_bytes()).unwrap();
        }
        msg_queue.clear();
    }
}

fn handle_client() {
    reconnect();

    loop {
        handle_stdin();
    }

}

fn handle_stdin() {
    let run_flag = &RUN_FLAG;
    let mut msg = String::new();
    stdin().read_line(&mut msg).unwrap();
    if !run_flag.load(Ordering::SeqCst) {
        MESSAGE_QUEUE.lock().unwrap().push(msg);

        reconnect();
        return;
    }
    let client = CLIENT.lock().unwrap();
    let mut stream = &client.as_ref().unwrap().stream;
    stream.write_all(msg.as_bytes()).unwrap();
    let mut count = HEARTBEAT_COUNT.lock().unwrap();
    *count = HEARTBEAT_INTERVAL;
}

fn handle_response() {
    let run_flag = &RUN_FLAG;
    loop {
        if !run_flag.load(Ordering::SeqCst) {
            break;
        }
        let client = CLIENT.lock().unwrap();
        let stream = &client.as_ref().unwrap().fork().stream;
        drop(client);
        let mut buf_reader = BufReader::new(stream.try_clone().unwrap());
        loop {
            let mut line = String::new();
            if let Ok(_) = buf_reader.read_line(&mut line) {
                if line == "0000\n" {
                    println!("End message");
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
        run_flag.store(false, Ordering::SeqCst);
    }
}

fn handle_heartbeat() {
    loop{
        thread::sleep(std::time::Duration::from_secs(1));
        let mut count = HEARTBEAT_COUNT.lock().unwrap();
        if *count > 0 {
            *count -= 1;
        } else {
            *count = HEARTBEAT_INTERVAL;
            let run_flag = &RUN_FLAG;
            if run_flag.load(Ordering::SeqCst) {
                let client = CLIENT.lock().unwrap();
                let mut stream = &client.as_ref().unwrap().stream;
                stream.write_all("\u{1F493}\n".as_bytes()).unwrap();
            } else {
                break;
            }
        }
    }
}
