use std::net::TcpStream;
use std::io::{BufRead, BufReader, Write};

fn main() {
    // 连接服务器
    let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    println!("Connected to server");

    // 向服务器发送消息
    let request = b"Hello, server!\nHello, server!\n0000\n";
    stream.write_all(request).unwrap();

    // 读取服务器的响应
    let buf_reader = BufReader::new(&mut stream);
    let req: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| *line != "0000")
        .collect();
    println!("Response: {:#?}", req);
}
