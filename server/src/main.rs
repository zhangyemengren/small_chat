use std::io::{BufRead, BufReader, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::{JoinHandle, ThreadId};
use std::time::Duration;

const MAX_CONNECT: usize = 2;

struct User {
    id: ThreadId,
    stream: TcpStream,
    name: String,
}

fn main() {
    // 监听
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    // 使用arc + mutex 代替全局变量
    let pool: Arc<Mutex<Vec<JoinHandle<()>>>> = Arc::new(Mutex::new(Vec::new()));
    let users: Arc<Mutex<Vec<User>>> = Arc::new(Mutex::new(Vec::new()));
    // 接收并处理每一个请求
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let pool_clone = pool.clone();
        let users_clone = users.clone();
        let mut pool = pool.lock().unwrap();
        if pool.len() >= MAX_CONNECT {
            println!("连接数过多");
            continue;
        }

        let handle = thread::spawn(move || {
            let name = stream.peer_addr().unwrap().to_string();
            println!("New connection: {}", name);
            let user = User {
                id: thread::current().id(),
                stream,
                name,
            };
            let mut users = users_clone.lock().unwrap();
            users.push(user);
            let users_clone = users_clone.clone();
            drop(users);
            handle_connection(users_clone);
            let mut pool = pool_clone.lock().unwrap();
            pool.retain(|x| x.thread().id() != thread::current().id());
        });

        pool.push(handle);
    }
}
// 处理请求 读取并返回
fn handle_connection(users: Arc<Mutex<Vec<User>>>) {
    let users = users.lock().unwrap();
    let user = users.iter().find(|x| x.id == thread::current().id());
    println!("id: {:?}",user.unwrap().id);
    let mut stream = user.unwrap().stream.try_clone().unwrap();
    let mut buf_reader = BufReader::new(stream.try_clone().unwrap());
    drop(users);
    loop {
        let mut line = String::new();
        if let Ok(_) = buf_reader.read_line(&mut line) {
            if line == "0000\n" {
                stream.write_all(b"finish messsage\n0000\n").unwrap();
                break;
            }
            if !line.is_empty() {
                let mut response = String::new();
                response = response + line.as_str();
                stream.write_all(response.as_bytes()).unwrap();
                println!("Request: {}", line);
                // 推送消息
                for x in 1..3 {
                    thread::sleep(Duration::from_secs(1));
                    let response = format!("times {}\n", x);
                    stream.write_all(response.as_bytes()).unwrap();
                }
            }
        } else {
            println!("Disconnected from server");
            break;
        }
    }
    println!("断开连接");
    stream.shutdown(Shutdown::Both).unwrap();
}
