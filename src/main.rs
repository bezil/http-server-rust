use std::{net::{TcpListener, TcpStream}, io::{Write, Read}, thread, fs::{read, File}, env};

fn request_file_content(path: &str, req_body: &str) -> String {
    let what = "application/octet-stream".to_string();
    let status = "201 OK".to_string();

    let mut root_path = "".to_string();
    let args: Vec<String> = env::args().collect();

    for (index, arg) in args.iter().enumerate() {
        if arg == "--directory" {
            root_path = match args.get(index + 1) {
                Some(value) => value.to_string(),
                None => "".to_string(),
            };
        }
    }
    let file_name = match path.strip_prefix("/files/") {
        Some(suffix) => suffix.to_string(),
        None => path.to_string(),
    };
    let file_path = format!("{}/{}", root_path, file_name);
    println!("file path: {}, root dir: {}, file name: {}", file_path, root_path, file_name);

    let body = req_body;

    let mut new_file = File::create(&file_path).expect("file creation failed");
    new_file.write_all(body.as_bytes()).expect("file write failed");

    return format!("HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}", status, what, body.as_bytes().len(), body.to_string());
}

fn response_content(content: &str) -> String {
    let what;
    let length;
    let mut body="".to_string();
    let status;
    let response;

    println!("path: {:?}", content);

    if content.contains("/files/") {
        what = "application/octet-stream".to_string();
        let mut root_path = "".to_string();
        let args: Vec<String> = env::args().collect();
        println!("args: {:?}", args);

        for (index, arg) in args.iter().enumerate() {
            if arg == "--directory" {
                root_path = match args.get(index + 1) {
                    Some(value) => value.to_string(),
                    None => "".to_string(),
                };
            }
        }
        let file_name = match content.strip_prefix("/files/") {
            Some(suffix) => suffix.to_string(),
            None => content.to_string(),
        };
        let file_path = format!("{}/{}", root_path, file_name);
        println!("file path: {}, root dir: {}, file name: {}", file_path, root_path, file_name);

        match read(&file_path) {
            Ok(file) => {
                body = String::from_utf8(file).unwrap();
            },
            Err(err) => {
                println!("Error opening file: {}", err);
                status = "404 Not Found".to_string();
                length = "".len().to_string();

                return format!("HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}", status, what, length, body);

            }
        };
        status = "200 OK".to_string();

        length = body.as_bytes().len().to_string();
        response = format!("HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}", status, what, length, body);

    } else {
        what = "text/plain".to_string();
        status = "200 OK".to_string();
        body = match content.strip_prefix("/echo/") {
            Some(suffix) => suffix.to_string(),
            None => content.to_string(),
        };
        length = body.as_bytes().len().to_string();
        response = format!("HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}", status, what, length, body);

    }

    response
}
fn response_header(header: &str) -> String {
    let what = "text/plain";
    let length = header.as_bytes().len();

    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}", what, length, header);
    response
}

fn handle_request(mut _stream: TcpStream) {
    let mut buffer = [0; 1024];

    println!("accepted connection");
    _stream.read(&mut buffer).expect("Read Error");
    let request_string = String::from_utf8_lossy(&buffer[..]);
    let store_lines: Vec<&str> = request_string.split("\r\n").collect();
    let path: Vec<&str> = store_lines[0].split(" ").collect();
    let mut agent_line = "".to_string();
    let mut agent_name = "".to_string();

    let binding = store_lines[store_lines.len() - 1].to_string();
    let request_body = binding.trim_matches('\0');

    for line in store_lines {
        if line.contains("User-Agent:") {
            agent_line = line.to_string();
            break;
        };
    }

    if agent_line.len() > 1 {
        let agent_pairs: Vec<&str> = agent_line.split(':').collect();
        agent_name = agent_pairs[1].to_string().trim_start().to_string();
    }

    let start_with = path[1].starts_with("/echo/") || path[1].starts_with("/files/");

    if path[0].contains("POST") && path[1].starts_with("/files/") {
        _stream.write_all(request_file_content(path[1], &request_body).as_bytes()).expect("Failed");

        let mut server_response = String::new();
        _stream.read_to_string(&mut server_response).expect("Server response failed");
    } else {
        match start_with {
            true => _stream.write_all(response_content(path[1]).as_bytes()).expect("Failed"),
            false => {
                match path[1] {
                    "/user-agent" => _stream.write_all(response_header(&agent_name).as_bytes()).expect("Failed"),
                    "/" => _stream.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).expect("Failed"),
                    _ => _stream.write_all("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes()).expect("Failed")
                }
            }
        }
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_request(stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
