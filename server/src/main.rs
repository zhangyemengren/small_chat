use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::{JoinHandle, ThreadId};
// use std::time::Duration;

const MAX_CONNECT: usize = 2;
const SHUTDOWN_INTERVAL: u64 = 10;

#[derive(Debug)]
struct User {
    id: ThreadId,
    stream: TcpStream,
    name: String,
}

fn main() {
    // 监听
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let pool: Arc<Mutex<Vec<JoinHandle<()>>>> = Arc::new(Mutex::new(Vec::new()));
    let users: Arc<Mutex<Vec<User>>> = Arc::new(Mutex::new(Vec::new()));
    // 接收并处理每一个请求
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(SHUTDOWN_INTERVAL)))
            .unwrap();
        let pool_clone = pool.clone();
        let users_clone = users.clone();
        let mut pool = pool.lock().unwrap();
        if pool.len() >= MAX_CONNECT {
            println!("Too many connections");
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
            drop(users);
            handle_connection(users_clone.clone());
            let mut pool = pool_clone.lock().unwrap();
            pool.retain(|x| x.thread().id() != thread::current().id());
            let mut users = users_clone.lock().unwrap();
            users.retain(|x| x.id != thread::current().id());
        });

        pool.push(handle);
    }
}
// 处理请求 读取并返回
fn handle_connection(users: Arc<Mutex<Vec<User>>>) {
    let users_guard = users.lock().unwrap();
    let user = users_guard
        .iter()
        .find(|x| x.id == thread::current().id())
        .unwrap();
    let user_name = user.name.clone();

    let mut stream = user.stream.try_clone().unwrap();
    let mut buf_reader = BufReader::new(stream.try_clone().unwrap());
    drop(users_guard);
    loop {
        let mut line = String::new();
        match buf_reader.read_line(&mut line) {
            Ok(0) => {
                // 流结束，客户端关闭了连接
                println!("Client disconnected");
                break;
            }
            Ok(_) => {
                if line == "0000\n" {
                    stream.write_all(b"End messsage\n0000\n").unwrap();
                    break;
                }
                if line == "\u{1F493}\n" {
                    println!("Heartbeat\u{1F493} from {}", user_name);
                    continue;
                }
                if !line.is_empty() {
                    let mut response = String::new();
                    response = response + line.as_str();
                    println!("Request: {} from {}", line, user_name);
                    // 推送消息
                    stream.write_all(response.as_bytes()).unwrap();
                    let users = users.lock().unwrap();
                    users
                        .iter()
                        .filter(|x| x.id != thread::current().id())
                        .map(|x| x.stream.try_clone().unwrap())
                        .for_each(|mut s| {
                            let msg = format!("{}: {}", user_name, line);
                            s.write_all(msg.as_bytes()).unwrap();
                        });
                }
            }
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut {
                    // 超时：SHUTDOWN_INTERVAL秒内没有收到数据
                    println!("Timed out due to inactivity");
                    break;
                } else {
                    // 其他错误
                    println!("Error occurred: {}", e);
                    break;
                }
            }
        }
    }
    println!("Disconnect loop end");
    stream.shutdown(Shutdown::Both).unwrap();
}
