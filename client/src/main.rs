use std::net::TcpStream;
use std::io::{BufRead, BufReader, stdin, Write};

fn main() {
    println!("input");
    loop {
        let mut msg = String::new();
        stdin().read_line(&mut msg).unwrap();
        send(msg);
    }
}
fn send(msg: String){
    // 连接服务器
    let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    println!("Connected to server");
    let msg = msg + "0000\n";
    // 向服务器发送消息
    stream.write_all(msg.as_bytes()).unwrap();

    // 读取服务器的响应
    let buf_reader = BufReader::new(&mut stream);
    let req: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| *line != "0000")
        .collect();
    println!("Response: {:#?}", req);
}
