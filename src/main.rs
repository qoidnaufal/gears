use std::{
    io::{BufRead, BufReader, Write},
    net::{SocketAddr, TcpListener, TcpStream},
};

use http_server::ThreadPool;

fn handle_read(mut stream: &TcpStream) -> Option<std::io::Result<String>> {
    let buf = BufReader::new(&mut stream);
    buf.lines().next()
}

fn handle_write(mut stream: &TcpStream, req_line: Option<std::io::Result<String>>) {
    if let Some(Ok(request_line)) = req_line {
        let (status_line, body) = match &request_line[..] {
            "GET / HTTP/1.1" => {
                let status_line = "HTTP/1.1 200 OK";
                let body = include_str!("../public/index.html");

                (status_line, body)
            }
            "GET /sleep HTTP/1.1" => {
                std::thread::sleep(std::time::Duration::from_secs(10));
                let status_line = "HTTP/1.1 200 OK";
                let body = include_str!("../public/index.html");

                (status_line, body)
            }
            _ => {
                let status_line = "HTTP/1.1 404 NOT FOUND";
                let body = "<p>Oopsie, We don't know what you're looking for!!!</p>";

                (status_line, body)
            }
        };

        let length = body.len();
        let response = format!(
            "{}\r\nContent-Length: {}\r\n\r\n{}",
            status_line, length, body
        );

        match stream.write_all(response.as_bytes()) {
            Ok(_) => (),
            Err(err) => eprintln!("Failed to write the response: {}", err),
        }
    }
}

fn handle_connection(stream: TcpStream) {
    let req_line = handle_read(&stream);
    handle_write(&stream, req_line);
}

fn main() -> std::io::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 6969));
    let listener = TcpListener::bind(addr)?;
    println!("Listening on: {}", addr);

    let pool = ThreadPool::new(4);

    for s in listener.incoming() {
        match s {
            Ok(stream) => {
                pool.execute(|| handle_connection(stream));
            }
            Err(err) => eprintln!("Connection refused: {}", err),
        }
    }

    Ok(())
}
