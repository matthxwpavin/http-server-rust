use std::io::Write;
#[allow(unused_imports)]
use std::net::TcpListener;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!!!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                _ = stream.write_all("HTTP/1.1 200 OK\r\n".as_bytes());
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
