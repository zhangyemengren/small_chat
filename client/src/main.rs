use std::io::{stdin, BufRead, BufReader, Write};
use std::net::TcpStream;
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

static CLIENT: Mutex<Option<Client>> = Mutex::new(None);
static RUN_FLAG: AtomicBool = AtomicBool::new(true);

fn main() {
    handle_client();
}

fn reconnect() {
    let mut client = CLIENT.lock().unwrap();
    *client = Some(Client::new("127.0.0.1:8080"));
    RUN_FLAG.store(true, Ordering::SeqCst);
}

fn handle_client() {
    reconnect();

    let _handle = thread::spawn(move || {
        handle_response();
    });

    loop {
        handle_stdin();
    }

}

fn handle_stdin() {
    let run_flag = &RUN_FLAG;
    let mut msg = String::new();
    stdin().read_line(&mut msg).unwrap();
    if !run_flag.load(Ordering::SeqCst) {
        println!("{} 该条消息应该推入消息队列 重连时直接发送", msg);
        reconnect();
        return;
    }
    let client = CLIENT.lock().unwrap();
    let mut stream = &client.as_ref().unwrap().stream;
    stream.write_all(msg.as_bytes()).unwrap();
}

fn handle_response() {
    let run_flag = &RUN_FLAG;
    loop {
        if !run_flag.load(Ordering::SeqCst) {
            continue;
        }
        let client = CLIENT.lock().unwrap();
        let stream = &client.as_ref().unwrap().fork().stream;
        drop(client);
        let mut buf_reader = BufReader::new(stream.try_clone().unwrap());
        loop {
            let mut line = String::new();
            if let Ok(_) = buf_reader.read_line(&mut line) {
                if line == "0000\n" {
                    println!("finish message");
                    run_flag.store(false, Ordering::SeqCst);
                    break;
                }
                if !line.is_empty() {
                    println!("Received: {}", line);
                }
            } else {
                println!("Disconnected from server");
                run_flag.store(false, Ordering::SeqCst);
                break;
            }
        }
    }
}
