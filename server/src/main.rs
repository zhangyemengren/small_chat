use std::io::{BufRead, BufReader, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

fn main() {
    // 监听
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    // 接收并处理每一个请求
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(move || {
            println!("New connection: {}", stream.peer_addr().unwrap());
            handle_connection(stream);
        });
    }
}
// 处理请求 读取并返回
fn handle_connection(mut stream: TcpStream) {
    let mut buf_reader = BufReader::new(stream.try_clone().unwrap());

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
