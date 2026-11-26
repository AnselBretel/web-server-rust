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

use color_eyre::eyre;
use reqwest::StatusCode;

async fn fetch_connection(url: &str) -> eyre::Result<(StatusCode, String)> {
    let response = reqwest::get(url).await?;
    let status = response.status();
    let body = response.text().await?;
    Ok((status, body))
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let url = "http://127.0.0.1:7878/";
    for _ in 0..10 {
        let (status, body) = fetch_connection(url).await?;
        println!("The status is: {status}");
        println!("The content is: {body}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn fetch_once_reads_status_and_body() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_task = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await.unwrap();

            let body = "<h1>Test OK</h1>";
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{body}",
                body.len()
            );
            socket.write_all(resp.as_bytes()).await.unwrap();
        });

        let url = format!("http://{addr}/");
        let (status, body) = fetch_connection(&url).await.unwrap();

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "<h1>Test OK</h1>");

        server_task.await.unwrap();
    }

    #[tokio::test]
    async fn fetch_once_fails_when_server_is_down() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);

        let url = format!("http://{addr}/");
        let res = fetch_connection(&url).await;
        assert!(res.is_err());
    }
}
