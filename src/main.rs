#[allow(unused_imports)]
use std::net::TcpListener;
use std::{
    io::{BufRead, BufReader, Read, Write},
    ops::Index,
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!!!");
    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let mut buffer = String::new();
                _ = match BufReader::new(&stream).read_line(&mut buffer) {
                    Ok(_) => {
                        if buffer.split(" ").collect::<Vec<&str>>()[1] == "/" {
                            stream
                                .write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes())
                        } else {
                            stream.write_all(
                                "HTTP/1.1 404 Not Found\r\n\r\n".as_bytes(),
                            )
                        }
                    }
                    Err(error) => {
                        println!("an error occurred: {error:?}");
                        stream.write_all(
                            "HTTP/1.1 500 Internal Server Error".as_bytes(),
                        )
                    }
                };
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
