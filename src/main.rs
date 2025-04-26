mod http_request;

use core::str;
use flate2::write::GzEncoder;
use flate2::Compression;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::exit;
use std::{env, fs};
use std::{thread, time};

use http_request::HttpRequest;

fn handle(
    stream: &mut TcpStream,
    dir: &str,
) -> (Vec<u8>, Option<String>, bool) {
    let mut buf = [0u8; 512];
    if let Err(err) = stream.read(&mut buf) {
        return (
            Vec::from(b"HTTP/1.1 500 Internal Server Error\r\n\r\n"),
            Some(format!("could not read: {err:?}")),
            false,
        );
    }

    let data = match str::from_utf8(&buf) {
        Ok(data) => data,
        Err(err) => {
            return (
                Vec::from(b"HTTP/1.1 400 Bad Request\r\n\r\n"),
                Some(format!("could not create a buf string: {err:?}")),
                false,
            );
        }
    };
    println!("{data}");

    let req = match HttpRequest::parse(data) {
        Some(req) => req,
        None => return (Vec::from(b"HTTP/1.1 200 OK\r\n"), None, false),
    };

    let connection_header = req.headers.get("Connection");
    let is_close = if let Some(value) = connection_header {
        value.iter().any(|v| v == "close")
    } else {
        false
    };

    let re = Regex::new(r"/echo/(?<echo_str>.+)").unwrap();

    if req.path == "/" {
        (Vec::from(b"HTTP/1.1 200 OK\r\n\r\n"), None, is_close)
    } else if re.is_match(&req.path) {
        let echo = re
            .captures(&req.path)
            .unwrap()
            .name("echo_str")
            .unwrap()
            .as_str();

        let mut buf: Vec<u8> = vec![];
        buf.extend_from_slice(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n",
        );

        let mut echo_compression: Option<Vec<u8>> = None;
        let mut content_lenght = echo.len();
        if let Some(values) = req.headers.get("Accept-Encoding") {
            if values.iter().any(|enc| enc == "gzip") {
                let mut e = GzEncoder::new(Vec::new(), Compression::default());
                e.write_all(echo.as_bytes()).unwrap();
                let encoded = e.finish().unwrap();
                content_lenght = encoded.len();
                echo_compression = Some(encoded);
                buf.extend_from_slice(b"Content-Encoding: gzip\r\n");
            }
        }

        buf.extend_from_slice(
            format!("Content-Length: {}\r\n\r\n", content_lenght).as_bytes(),
        );

        match echo_compression {
            None => buf.extend_from_slice(echo.as_bytes()),
            Some(compression) => buf.extend_from_slice(&compression),
        };

        (buf, None, is_close)
    } else if req.path == "/user-agent" {
        let user_agent = &req.headers.get("User-Agent").unwrap()[0];

        (
            Vec::from(
                format!(
                    "\
                                    HTTP/1.1 200 OK\r\n\
                                    Content-Type: text/plain\r\n\
                                    Content-Length: {}\r\n\
                                    \r\n\
                                    {}",
                    user_agent.len(),
                    user_agent
                )
                .as_bytes(),
            ),
            None,
            is_close,
        )
    } else if req.path.starts_with("/files/") {
        let filename = req.path.trim_start_matches("/files/");
        let dir = if dir.is_empty() { "/tmp/" } else { dir };
        let filename = &format!("{dir}{filename}");

        if req.method == "POST" {
            println!("creating a file: {filename}");
            match req.body {
                None => (
                    Vec::from(b"HTTP/1.1 400 Bad Request\r\n\r\n"),
                    Some(String::from("no payload found")),
                    is_close,
                ),
                Some(mut content) => {
                    content = content.replace('\x00', "");
                    if let Err(err) = fs::write(filename, content) {
                        (
                            Vec::from(b"HTTP/1.1 400 Bad Request\r\n\r\n"),
                            Some(format!("could not write file: {err:?}")),
                            is_close,
                        )
                    } else {
                        (
                            Vec::from(b"HTTP/1.1 201 Created\r\n\r\n"),
                            None,
                            is_close,
                        )
                    }
                }
            }
        } else {
            println!("reading a file: {filename}",);
            match fs::read(filename) {
                Err(err) => {
                    if err.kind() == ErrorKind::NotFound {
                        (
                            Vec::from(b"HTTP/1.1 404 Not Found\r\n\r\n"),
                            Some(format!(
                                "no file found, filename: {filename}"
                            )),
                            is_close,
                        )
                    } else {
                        (
                            Vec::from(b"HTTP/1.1 400 Bad Request\r\n\r\n"),
                            Some(format!("could not read a file: {err:?}")),
                            is_close,
                        )
                    }
                }
                Ok(content) => {
                    let content =
                        str::from_utf8(&content).unwrap().replace("\x00", "");
                    (
                        Vec::from(
                            format!(
                                "\
                    HTTP/1.1 200 OK\r\n\
                    Content-Type: application/octet-stream\r\n\
                    Content-Length: {}\r\n\
                    \r\n\
                    {}
                    ",
                                content.len(),
                                content,
                            )
                            .as_bytes(),
                        ),
                        None,
                        is_close,
                    )
                }
            }
        }
    } else {
        (Vec::from(b"HTTP/1.1 404 Not Found\r\n\r\n"), None, is_close)
    }
}

const ARGS: [&str; 1] = ["--directory"];

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!!!");
    // Uncomment this block to pass the first stage

    let args_set = HashSet::from(ARGS);
    let args: Vec<String> = env::args().collect();
    let mut passed_args: HashMap<String, String> = HashMap::new();
    for (i, arg) in args[1..].iter().enumerate() {
        // TODO: args_set to HashMap to map with its properties such as, is option value need?
        if i % 2 == 1 {
            continue;
        }
        if !args_set.contains(arg.as_str()) {
            eprintln!("unknown an option: {arg}");
            exit(-1);
        }
        let i = args.iter().position(|v| v == arg).unwrap();
        let value = &args[i + 1];
        passed_args.insert(arg.clone(), value.clone());
    }

    let dir_key = &String::from(ARGS[0]);
    let mut dir = String::new();
    if passed_args.contains_key(dir_key) {
        let arg_dir = &passed_args.get(dir_key).unwrap().clone();
        println!("creating a directory: {arg_dir}");
        if let Err(err) = fs::create_dir_all(arg_dir) {
            eprintln!(
                "could not create a directory: dir: {arg_dir}, error: {err}"
            );
            exit(-1);
        }
        dir = arg_dir.clone();
    }

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        println!("accpeting a new connection...");
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("an error occurred: {err:?}");
                continue;
            }
        };

        let dir_cloned = dir.clone();
        std::thread::spawn(move || loop {
            let (mut response, error, is_close) =
                handle(&mut stream, &dir_cloned);
            if let Some(error) = error {
                eprintln!("{error}");
            }
            if is_close {
                let response_str = String::from_utf8(response.clone()).unwrap();
                let splited: Vec<&str> =
                    response_str.split("\r\n\r\n").collect();
                response = Vec::from(format!(
                    "{}\r\nConnection: close\r\n\r\n{}",
                    splited[0], splited[1],
                ));
            }
            println!("{}", String::from_utf8(response.clone()).unwrap());

            _ = stream.write_all(response.as_slice());
            // _ = stream.flush();
            if is_close {
                break;
            }

            let ten_millis = time::Duration::from_millis(500);
            thread::sleep(ten_millis);
        });
    }
}
