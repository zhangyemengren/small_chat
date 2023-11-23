use std::net::TcpStream;
use std::io::{BufRead, BufReader, stdin, Write};
use std::thread;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    let stream_fork = stream.try_clone().unwrap();
    println!("input");
    thread::spawn(move || {
        handle_response(stream_fork);
    });
    loop {
        let mut msg = String::new();
        stdin().read_line(&mut msg).unwrap();
        send(&mut stream, msg);
    }
}
fn send(stream: &mut TcpStream, msg: String){
    // 连接服务器
    println!("Connected to server");
    // 向服务器发送消息
    stream.write_all(msg.as_bytes()).unwrap();
}
fn handle_response(stream: TcpStream) {
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
}
