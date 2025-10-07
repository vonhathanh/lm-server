use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

const SERVER_ADDRESS: &str = "127.0.0.1:8000";

fn main() {
    let listener = TcpListener::bind(SERVER_ADDRESS).unwrap();
    println!("Server is listening at: {SERVER_ADDRESS}");
    for stream in listener.incoming() {
        let tcp_stream = stream.unwrap();
        handle_connection(tcp_stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    println!("Connection estalished");
    let buf_reader = BufReader::new(&stream);
    let first_line = buf_reader.lines().next().unwrap().unwrap();

    println!("request path: {first_line}");

    let (status_line, content) = if first_line == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", "{'message': 'hello world'}")
    }else {
        ("HTTP/1.1 404 NOT FOUND", "{'message': 'not found'}")
    };

    if first_line == "GET / HTTP/1.1" {
        let response = format!("{status_line}\r\nContent-Length: {}\r\n\r\n{content}", content.len());
        stream.write_all(response.as_bytes()).unwrap();
    } else {
        let response = format!("{status_line}\r\nContent-Length: {}\r\n\r\n{content}", content.len());
        stream.write_all(response.as_bytes()).unwrap();
    }
}
