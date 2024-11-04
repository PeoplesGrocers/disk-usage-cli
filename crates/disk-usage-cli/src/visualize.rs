// showing disk usage with .gitignore awareness
// Copyright (C) 2024 Peoples Grocers LLC
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
use serde::Serialize;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::PathBuf;

const HTML_CONTENT: &str = include_str!(concat!(env!("OUT_DIR"), "/index.html"));
const JS_CONTENT: &str = include_str!(concat!(env!("OUT_DIR"), "/index.js"));
const CSS_CONTENT: &str = include_str!(concat!(env!("OUT_DIR"), "/index.css"));

#[derive(Serialize)]
struct Metafile {
    inputs: HashMap<String, InputFile>,
    outputs: HashMap<String, OutputFile>,
}

#[derive(Serialize)]
struct InputFile {
    bytes: u64,
    imports: Vec<ImportRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<&'static str>,
}

#[derive(Serialize)]
struct OutputFile {
    bytes: u64,
    inputs: HashMap<String, InputForOutput>,
    imports: Vec<ImportRecord>,
    exports: Vec<String>,
}

#[derive(Serialize)]
struct ImportRecord {
    path: String,
    kind: String,
}

#[allow(non_snake_case)]
#[derive(Serialize)]
struct InputForOutput {
    bytesInOutput: u64,
}

pub fn encode(entries: &Vec<(PathBuf, (u64, u64, bool))>) -> String {
    let mut total_size = 0;
    let mut inputs = HashMap::new();
    let mut output_inputs = HashMap::new();

    // Since groups is already sorted by prefix, we can look ahead to check for children
    for i in 0..entries.len() {
        let (ref current_path, (not_ignored_size, ignored_size, _is_file)) = entries[i];
        let current_str = current_path.to_string_lossy();

        // Skip empty paths
        if current_str.is_empty() {
            continue;
        }

        // Look ahead to see if this is a prefix of the next path
        let is_prefix = entries
            .get(i + 1)
            .map(|(next_path, _)| next_path.starts_with(current_path))
            .unwrap_or(false);

        let size = not_ignored_size + ignored_size;
        // Only process paths that aren't prefixes of later paths (leaf nodes)
        if !is_prefix {
            inputs.insert(
                current_str.to_string(),
                InputFile {
                    bytes: size,
                    imports: vec![],
                    format: if ignored_size > 0 && not_ignored_size > 0 {
                        Some("both")
                    } else if ignored_size > 0 {
                        Some("cjs")
                    } else if not_ignored_size > 0 {
                        Some("esm")
                    } else {
                        None
                    },
                },
            );

            output_inputs.insert(
                current_str.to_string(),
                InputForOutput {
                    bytesInOutput: size,
                },
            );

            total_size += size;
        }
    }

    let mut outputs = HashMap::new();
    outputs.insert(
        "root".to_string(),
        OutputFile {
            bytes: total_size,
            inputs: output_inputs,
            imports: vec![],
            exports: vec![],
        },
    );

    let metafile = Metafile { inputs, outputs };

    let json = serde_json::to_string_pretty(&metafile).unwrap();
    json
}

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

pub fn view_in_browser(entries: &Vec<(PathBuf, (u64, u64, bool))>, open_in_browser: bool) {
    for port in 8001..65535 {
        let address = format!("127.0.0.1:{}", port);
        match TcpListener::bind(&address) {
            Err(e) if e.kind() == ErrorKind::AddrInUse => {
                eprintln!("Port {} is in use, trying the next one", port);
                continue;
            }
            Err(e) => panic!("Failed to bind: {}", e),
            Ok(listener) => {
                eprintln!("Server running on http://{}", &address);
                let json = encode(entries);

                if open_in_browser {
                    open::that_detached(format!("http://{}", address)).unwrap();
                }

                for stream in listener.incoming() {
                    match stream {
                        Ok(stream) => handle_connection(stream, &json),
                        Err(e) => eprintln!("Connection failed: {}", e),
                    }
                }

                break;
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, payload: &str) {
    let mut buffer = [0; 256 * 1024];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer[..]);
    let request_line = request.lines().next().unwrap_or("");

    let (status_line, content, content_type) = match request_line {
        req if req.starts_with("GET / ") => ("HTTP/1.1 200 OK", HTML_CONTENT, "text/html"),
        req if req.starts_with("GET /index.js ") => {
            ("HTTP/1.1 200 OK", JS_CONTENT, "application/javascript")
        }
        req if req.starts_with("GET /index.css ") => ("HTTP/1.1 200 OK", CSS_CONTENT, "text/css"),
        req if req.starts_with("GET /metafile.json ") => {
            ("HTTP/1.1 200 OK", payload, "application/json")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404 Not Found", "text/plain"),
    };

    let headers = [
        format!("Content-Length: {}", content.len()),
        format!("Content-Type: {}", content_type),
        // Security headers allow using high precision timing APIs in the browser
        "Cross-Origin-Opener-Policy: same-origin".to_owned(),
        "Cross-Origin-Embedder-Policy: require-corp".to_owned(),
        "Cross-Origin-Resource-Policy: same-origin".to_owned(),
        "Timing-Allow-Origin: *".to_owned(),
    ]
    .join("\r\n");
    let response = format!("{}\r\n{}\r\n\r\n{}", status_line, headers, content);

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
