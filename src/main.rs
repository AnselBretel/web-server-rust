use std::{
    io::{BufRead, BufReader, prelude::*}, net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap(); // Like ::new creates a new TcpListener and binds it to the specified address and port
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        // Handle the client connection
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    // Here you can read from and write to the stream
    // For example, you can read data sent by the client and respond accordingly
    let buf_read = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_read
        .lines() //We extract the lines incoming from the stream and collect them into a vector of strings
        .map(|line| line.unwrap()) //We unwrap the Result<String, Error> to get the actual string
        .take_while(|line| !line.is_empty()) //we take lines until we encounter an empty line, which indicates the end of the HTTP request 
        .collect(); //we collect the lines into a vector of strings
    println!("Request: {http_request:#?}"); //then we print the request to the console 
}
