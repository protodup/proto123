use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Command, ExitCode};
use std::thread;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

/// Parses a raw HTTP/1.x response into status code, lower-cased header keys, and body.
pub fn parse_http_response(raw: &str) -> Result<HttpResponse, String> {
    let (head, body) = raw
        .split_once("\r\n\r\n")
        .or_else(|| raw.split_once("\n\n"))
        .unwrap_or((raw, ""));
    let mut lines = head.lines();
    let status_line = lines
        .next()
        .ok_or_else(|| "missing HTTP status line".to_owned())?;
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .ok_or_else(|| format!("invalid HTTP status line: {status_line:?}"))?;
    let mut headers = HashMap::new();
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_owned());
        }
    }
    Ok(HttpResponse {
        status,
        headers,
        body: body.to_owned(),
    })
}

/// Finds the first `.wasm` asset path referenced in HTML (e.g. `/proto123-wasm-abc_bg.wasm`).
pub fn find_wasm_href(html: &str) -> Option<String> {
    for token in html.split(['"', '\'', '(', ')', ' ', '\n', '\r', '\t']) {
        let trimmed = token.trim();
        if trimmed.ends_with(".wasm") && !trimmed.is_empty() {
            if trimmed.starts_with('/') {
                return Some(trimmed.to_owned());
            }
            return Some(format!("/{trimmed}"));
        }
    }
    None
}

struct ContainerGuard {
    id: String,
}

impl Drop for ContainerGuard {
    fn drop(&mut self) {
        let _ = Command::new("docker").args(["stop", &self.id]).output();
    }
}

fn http_get(addr: &str, path: &str) -> Result<HttpResponse, String> {
    let mut last_err = None;
    for _ in 0..20 {
        match TcpStream::connect(addr) {
            Ok(mut stream) => {
                let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                let req =
                    format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
                stream
                    .write_all(req.as_bytes())
                    .map_err(|e| format!("write failed: {e}"))?;
                let mut bytes = Vec::new();
                stream
                    .read_to_end(&mut bytes)
                    .map_err(|e| format!("read failed: {e}"))?;
                let text = String::from_utf8_lossy(&bytes);
                return parse_http_response(&text);
            }
            Err(e) => {
                last_err = Some(e);
                thread::sleep(Duration::from_millis(150));
            }
        }
    }
    Err(format!(
        "could not connect to {addr}: {}",
        last_err.map_or_else(|| "unknown".to_owned(), |e| e.to_string())
    ))
}

pub fn run() -> ExitCode {
    match Command::new("docker").arg("--version").output() {
        Ok(out) if out.status.success() => {}
        _ => {
            eprintln!("docker-check: `docker` not found");
            return ExitCode::from(2);
        }
    }

    match Command::new("docker")
        .args(["build", "-t", "proto123:check", "."])
        .status()
    {
        Ok(st) if st.success() => {}
        Ok(_) => {
            eprintln!("docker-check: `docker build` failed");
            return ExitCode::from(1);
        }
        Err(e) => {
            eprintln!("docker-check: failed to run docker build: {e}");
            return ExitCode::from(2);
        }
    }

    let run_out = match Command::new("docker")
        .args([
            "run",
            "-d",
            "--rm",
            "-p",
            "127.0.0.1:0:80",
            "proto123:check",
        ])
        .output()
    {
        Ok(o) if o.status.success() => o,
        Ok(o) => {
            eprintln!(
                "docker-check: `docker run` failed: {}",
                String::from_utf8_lossy(&o.stderr)
            );
            return ExitCode::from(1);
        }
        Err(e) => {
            eprintln!("docker-check: failed to run docker: {e}");
            return ExitCode::from(2);
        }
    };
    let container_id = String::from_utf8_lossy(&run_out.stdout).trim().to_owned();
    let _guard = ContainerGuard {
        id: container_id.clone(),
    };

    let port_out = match Command::new("docker")
        .args(["port", &container_id, "80"])
        .output()
    {
        Ok(o) if o.status.success() => o,
        Ok(o) => {
            eprintln!(
                "docker-check: `docker port` failed: {}",
                String::from_utf8_lossy(&o.stderr)
            );
            return ExitCode::from(1);
        }
        Err(e) => {
            eprintln!("docker-check: failed to query docker port: {e}");
            return ExitCode::from(2);
        }
    };
    let port_line = String::from_utf8_lossy(&port_out.stdout);
    let Some(host_addr) = port_line.lines().next().map(str::trim) else {
        eprintln!("docker-check: empty `docker port` output");
        return ExitCode::from(1);
    };

    let index = match http_get(host_addr, "/") {
        Ok(r) => r,
        Err(e) => {
            eprintln!("docker-check: GET / failed: {e}");
            return ExitCode::from(1);
        }
    };
    if index.status != 200 {
        eprintln!("docker-check: GET / returned status {}", index.status);
        return ExitCode::from(1);
    }

    let wasm_path = find_wasm_href(&index.body).or_else(|| {
        std::fs::read_dir(Path::new("dist"))
            .ok()?
            .flatten()
            .find(|e| e.path().extension().is_some_and(|x| x == "wasm"))
            .and_then(|e| e.file_name().into_string().ok())
            .map(|name| format!("/{name}"))
    });
    let Some(wasm_path) = wasm_path else {
        eprintln!("docker-check: could not locate .wasm asset in index HTML");
        return ExitCode::from(1);
    };

    let wasm_resp = match http_get(host_addr, &wasm_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("docker-check: GET {wasm_path} failed: {e}");
            return ExitCode::from(1);
        }
    };
    if wasm_resp.status != 200 {
        eprintln!(
            "docker-check: GET {wasm_path} returned {}",
            wasm_resp.status
        );
        return ExitCode::from(1);
    }
    let content_type = wasm_resp
        .headers
        .get("content-type")
        .map(String::as_str)
        .unwrap_or("");
    if !content_type.starts_with("application/wasm") {
        eprintln!(
            "docker-check: GET {wasm_path} Content-Type={content_type:?}, expected application/wasm"
        );
        return ExitCode::from(1);
    }

    println!("docker-check: ok ({wasm_path} Content-Type: {content_type})");
    ExitCode::SUCCESS
}
