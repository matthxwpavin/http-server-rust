use regex::Regex;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!!!");
    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    let re = Regex::new(r"/echo/(?<echo_str>.+)").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let mut bufs = Vec::new();
                _ = match BufReader::new(&stream).read_vectored(&mut bufs) {
                    Ok(_) => {
                        let path = "";
                        // let path = buffer.split(" ").collect::<Vec<&str>>()[1];
                        if path == "/" {
                            stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n")
                        } else if re.is_match(path) {
                            let echo = re
                                .captures(path)
                                .unwrap()
                                .name("echo_str")
                                .unwrap()
                                .as_str();
                            stream.write_all(
                                format!(
                                    "\
                                HTTP/1.1 200 OK\r\n\
                                Content-Type: text/plain\r\n\
                                Content-Length: {}\r\n\
                                \r\n{}",
                                    echo.len(),
                                    echo,
                                )
                                .as_bytes(),
                            )
                        } else if path == "/user-agent" {
                            stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n")
                        } else {
                            stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")
                        }
                    }
                    Err(error) => {
                        println!("an error occurred: {error:?}");
                        stream.write_all(b"HTTP/1.1 500 Internal Server Error")
                    }
                };
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
