use std::net::TcpStream;
use std::io::{BufRead, BufReader, stdin, Write};

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    println!("input");
    loop {
        let mut msg = String::new();
        stdin().read_line(&mut msg).unwrap();
        send(&mut stream, msg);
    }
}
fn send(stream: &mut TcpStream, msg: String){
    // 连接服务器
    // let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    println!("Connected to server");
    let msg = msg + "0000\n";
    // 向服务器发送消息
    stream.write_all(msg.as_bytes()).unwrap();

    // 读取服务器的响应
    let mut buf_reader = BufReader::new(stream);
    loop {
        let mut line = String::new();
        if let Ok(_) = buf_reader.read_line(&mut line) {
            if line == "0000\n" {
                break;
            }
            if !line.is_empty() {
                print!("Received: {}", line);
            }
        } else {
            println!("Disconnected from server");
            break;
        }
    }
    println!("over");
}
