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
        handle_connection(stream);
    }
}
// 处理请求 读取并返回
fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    // 逐行读取 并定义0000\n为结束符
    let req: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| *line != "0000")
        .collect();
    println!("Request: {:#?}", req);

    let mut response = String::new();
    response = response + req.join("\n").as_str() + "\n";
    stream.write_all(response.as_bytes()).unwrap();
    // 推送消息
    for x in 1..4 {
        thread::sleep(Duration::from_secs(1));
        let response = format!("times {}\n", x);
        stream.write_all(response.as_bytes()).unwrap();
    }
    // 结束链接
    let end = b"0000\n";
    stream.write_all(end).unwrap();
    stream.shutdown(Shutdown::Both).unwrap();
}
