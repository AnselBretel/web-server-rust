// Copyright 2026 SECO Mind Srl
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

use tokio::{
    fs,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    time::{Duration, sleep},
};

use color_eyre::eyre;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let listener = TcpListener::bind("127.0.0.1:7878").await?;
    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            match handle_connection(socket).await {
                Err(error) => eyre::bail!("{error}"),
                Ok(()) => {
                    println!("Connected to the ip address!");
                    Ok(())
                }
            }
        });
    }
}
async fn handle_connection(mut socket: TcpStream) -> eyre::Result<()> {
    let request_line = BufReader::new(&mut socket).lines().next_line().await?;

    let (status_line, filename) = match request_line.as_deref() {
        Some("GET / HTTP/1.1") => ("HTTP/1.1 200 OK", "html/hello.html"),
        Some("GET /sleep HTTP/1.1") => {
            sleep(Duration::from_secs(5)).await;
            ("HTTP/1.1 200 OK", "html/hello.html")
        }
        Some(_) => ("HTTP/1.1 404 NOT FOUND", "html/404.html"),
        None => return Ok(()),
    };
    let contents = fs::read_to_string(filename).await?;
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    socket.write_all(response.as_bytes()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;

    async fn run_request(raw_request: &str) -> eyre::Result<String> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;

        let server_task = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            handle_connection(socket).await.unwrap();
        });

        let mut client = TcpStream::connect(addr).await?;
        client.write_all(raw_request.as_bytes()).await?;
        client.shutdown().await?;

        let mut response = String::new();
        client.read_to_string(&mut response).await?;

        server_task.await?;
        Ok(response)
    }

    #[tokio::test]
    async fn returns_200_and_hello_html() {
        let expected_body = fs::read_to_string("html/hello.html").await.unwrap();
        let expected = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{expected_body}",
            expected_body.len()
        );

        let response = run_request("GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .await
            .unwrap();

        assert_eq!(response, expected);
    }

    #[tokio::test]
    async fn returns_404() {
        let expected_body = fs::read_to_string("html/404.html").await.unwrap();
        let expected = format!(
            "HTTP/1.1 404 NOT FOUND\r\nContent-Length: {}\r\n\r\n{expected_body}",
            expected_body.len()
        );

        let response = run_request("GET /non-existent HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .await
            .unwrap();

        assert_eq!(response, expected);
    }

    #[tokio::test]
    async fn empty_connection_returns_no_response() {
        let response = run_request("").await.unwrap();
        assert_eq!(response, "");
    }
}
