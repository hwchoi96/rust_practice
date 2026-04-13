//! 웹소켓 client side — `websocket_server` 와 짝

use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::time::Duration;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::time::interval;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Deserialize)]
struct WelcomeMessage {
    msg: String,
}

#[tokio::main]
async fn main() {
    run_client().await.expect("웹소켓 클라이언트 실행 실패.");
}

async fn run_client() -> Result<(), Box<dyn std::error::Error>> {
    let url = "ws://127.0.0.1:9999";
    let (mut ws, _resp) = connect_async(url).await?;

    println!("연결됨: {}", url);

    // 서버 환영 메시지 한 번 수신 (split 전에 처리)
    if let Some(first) = ws.next().await {
        let msg = first?;
        if let Message::Text(text) = msg {
            println!("서버 → {}", text);
            if let Ok(w) = serde_json::from_str::<WelcomeMessage>(&text) {
                println!("환영 메시지: {}", w.msg);
            }
        } else {
            println!("서버 → {:?}", msg);
        }
    }

    let (mut write, mut read) = ws.split();

    let stdin = io::stdin();
    let mut stdin = BufReader::new(stdin);
    let mut line = String::new();

    // 핑퐁을 위한, ping interval 설정
    let mut ping_interval = interval(Duration::from_secs(15));
    ping_interval.tick().await;

    println!("문자열 입력 후 Enter 로 전송. 종료: quit / exit (또는 Ctrl+D)");
    println!("────────────────────────────────────────");

    loop {
        line.clear();
        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(t))) => println!("[서버] {}", t),
                    Some(Ok(Message::Close(f))) => {
                        println!("[서버] 연결 종료: {:?}", f);
                        return Ok(());
                    }
                    Some(Ok(other)) => println!("[서버] {:?}", other),
                    Some(Err(e)) => {
                        eprintln!("[수신 오류] {}", e);
                        break;
                    }
                    None => {
                        println!("[연결 끊김]");
                        break;
                    }
                }
            }
            n = stdin.read_line(&mut line) => {
                if n? == 0 {
                    println!("(표준입력 종료)");
                    break;
                }
                let text = line.trim_end_matches(['\r', '\n']).trim();
                if text.eq_ignore_ascii_case("quit") || text.eq_ignore_ascii_case("exit") {
                    println!("종료합니다.");
                    break;
                }
                if text.is_empty() {
                    continue;
                }
                write.send(Message::Text(text.into())).await?;
            }
            _ = ping_interval.tick() => {
                println!("핑 전송!");
                write.send(Message::Ping(Vec::new())).await?;
            }
        }
    }

    let _ = write.send(Message::Close(None)).await;
    let _ = write.close().await;

    Ok(())
}
