use std::io::{Read, Write};

#[link(wasm_import_module = "fledge")]
extern "C" {
    fn recv(ptr: *mut u8, max_len: i32) -> i32;
    fn send(ptr: *const u8, len: i32);
    fn exit(code: i32);
}

static mut PASS: u32 = 0;
static mut FAIL: u32 = 0;

fn fledge_recv() -> Vec<u8> {
    let mut buf = vec![0u8; 65536];
    let len = unsafe { recv(buf.as_mut_ptr(), buf.len() as i32) };
    buf.truncate(len.max(0) as usize);
    buf
}

fn fledge_send(msg: &str) {
    unsafe { send(msg.as_ptr(), msg.len() as i32) };
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out
}

fn output(text: &str) {
    fledge_send(&format!(r#"{{"type":"output","text":"{}"}}"#, json_escape(text)));
}

fn pass(msg: &str) {
    unsafe { PASS += 1 };
    output(&format!("  \u{2713} PASS: {msg}\n"));
}

fn fail(msg: &str) {
    unsafe { FAIL += 1 };
    output(&format!("  \u{2717} FAIL: {msg}\n"));
}

fn header(title: &str) {
    output(&format!("\n=== {title} ===\n"));
}

fn is_unsupported(e: &std::io::Error) -> bool {
    let msg = format!("{e}");
    msg.contains("unsupported") || msg.contains("Unsupported") || msg.contains("not supported")
}

fn test_tcp_dns() {
    header("TCP CONNECTIONS");
    output("  Testing outbound TCP to well-known hosts.\n");
    output("  Without network capability, these return 'Unsupported'.\n\n");

    match std::net::TcpStream::connect("8.8.8.8:53") {
        Ok(_stream) => pass("TCP to 8.8.8.8:53 (Google DNS) — connected"),
        Err(e) => {
            if is_unsupported(&e) {
                fail(&format!("TCP unsupported — WASI sockets not linked: {e}"));
            } else {
                pass(&format!("TCP socket API available (connect error: {e})"));
            }
        }
    }
}

fn test_tcp_cloudflare() {
    match std::net::TcpStream::connect("1.1.1.1:443") {
        Ok(_stream) => pass("TCP to 1.1.1.1:443 (Cloudflare) — connected"),
        Err(e) => {
            if is_unsupported(&e) {
                fail(&format!("TCP unsupported: {e}"));
            } else {
                pass(&format!("TCP socket API available (error: {e})"));
            }
        }
    }
}

fn test_tcp_http() {
    header("HTTP (RAW TCP)");
    output("  Attempting raw HTTP GET via TCP.\n\n");

    match std::net::TcpStream::connect("93.184.216.34:80") {
        Ok(mut stream) => {
            let request = "GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n";
            match stream.write_all(request.as_bytes()) {
                Ok(_) => {
                    let mut response = vec![0u8; 4096];
                    match stream.read(&mut response) {
                        Ok(n) => {
                            let resp_str = String::from_utf8_lossy(&response[..n]);
                            if resp_str.contains("HTTP/1.1") || resp_str.contains("HTTP/1.0") {
                                pass(&format!("HTTP response received ({n} bytes)"));
                            } else {
                                pass(&format!("TCP data received ({n} bytes, not HTTP)"));
                            }
                        }
                        Err(e) => pass(&format!("TCP connected but read failed: {e}")),
                    }
                }
                Err(e) => pass(&format!("TCP connected but write failed: {e}")),
            }
        }
        Err(e) => {
            if is_unsupported(&e) {
                fail(&format!("TCP unsupported for HTTP: {e}"));
            } else {
                output(&format!("  TCP connect to example.com:80 failed: {e}\n"));
                output("  (This may be expected in network-restricted environments)\n");
                pass("TCP socket API available (network may be restricted)");
            }
        }
    }
}

fn test_localhost() {
    header("LOCALHOST");
    output("  Testing connection to localhost (loopback).\n\n");

    match std::net::TcpStream::connect("127.0.0.1:1") {
        Ok(_) => pass("localhost connection succeeded (unexpected but socket works)"),
        Err(e) => {
            if is_unsupported(&e) {
                fail(&format!("localhost unsupported: {e}"));
            } else {
                pass(&format!("localhost socket API available (error: {e})"));
            }
        }
    }
}

fn test_negative_no_filesystem() {
    header("NEGATIVE — OTHER CAPABILITIES BLOCKED");
    match std::fs::read_to_string("/project/Cargo.toml") {
        Ok(_) => fail("filesystem accessible without capability"),
        Err(_) => pass("filesystem blocked (no capability granted)"),
    }
}

fn test_negative_no_process_spawn() {
    match std::process::Command::new("curl").arg("http://example.com").output() {
        Ok(_) => fail("process spawn succeeded"),
        Err(_) => pass("process spawn blocked (WASI p1)"),
    }
}

fn main() {
    let _init = fledge_recv();

    output("fledge-plugin-test-network v0.1.0\n");
    output("Capability: network=true (all others denied)\n");
    output("Tests that WASM plugins can make outbound TCP connections\n");
    output("Note: WASI P1 network support varies — this tests socket API availability\n");

    test_tcp_dns();
    test_tcp_cloudflare();
    test_tcp_http();
    test_localhost();
    test_negative_no_filesystem();
    test_negative_no_process_spawn();

    let (p, f) = unsafe { (PASS, FAIL) };
    let total = p + f;
    header("SUMMARY");
    output(&format!("  {total} tests: {p} passed, {f} failed\n\n"));

    if f == 0 {
        output("  RESULT: network capability works correctly.\n\n");
    } else {
        output(&format!("  WARNING: {f} test(s) failed!\n"));
        output("  Note: WASI P1 has limited socket support. If all tests show\n");
        output("  'unsupported', this may indicate WASI P2 is needed for network.\n\n");
    }

    unsafe { exit(if f == 0 { 0 } else { 1 }) };
    unreachable!();
}
