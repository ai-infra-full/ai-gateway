use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

struct GatewayProcess(Child);

impl Drop for GatewayProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn provider_routes_echo_request_body() {
    let address = available_address();
    let child = Command::new(env!("CARGO_BIN_EXE_gateway"))
        .env("GATEWAY_ADDRESS", address.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("gateway should start");
    let mut gateway = GatewayProcess(child);

    wait_until_ready(&mut gateway.0, address);

    for path in [
        "/openai/v1/chat/completions",
        "/gemini/v1beta/models/gemini-pro:generateContent",
        "/anthropic/v1/messages",
    ] {
        assert_echo(address, path, "hello gateway");
    }
}

fn available_address() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").expect("should reserve a local port");
    listener
        .local_addr()
        .expect("should read the local address")
}

fn wait_until_ready(child: &mut Child, address: SocketAddr) {
    let deadline = Instant::now() + Duration::from_secs(5);

    while Instant::now() < deadline {
        if TcpStream::connect(address).is_ok() {
            return;
        }

        if let Some(status) = child.try_wait().expect("should inspect gateway process") {
            panic!("gateway exited before becoming ready: {status}");
        }

        thread::sleep(Duration::from_millis(25));
    }

    panic!("gateway did not listen on {address}");
}

fn assert_echo(address: SocketAddr, path: &str, body: &str) {
    let mut stream = TcpStream::connect(address).expect("should connect to gateway");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("should configure read timeout");

    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {address}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(request.as_bytes())
        .expect("should send request");

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .expect("should read response");

    let (head, response_body) = response
        .split_once("\r\n\r\n")
        .expect("response should contain headers");
    assert!(head.starts_with("HTTP/1.1 200 OK\r\n"), "{response}");
    assert_eq!(response_body, body);
}
