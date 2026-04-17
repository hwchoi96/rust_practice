//! 웹소켓 client side — `websocket_server` 와 짝

use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::time::Duration;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio::time::interval;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Deserialize)]
struct WelcomeMessage {
    msg: String,
}

#[tokio::main]
async fn main() {
    let (signal_tx, signal_rx) = mpsc::channel::<String>(16);

    // main에서 만든 송신자로 다른 태스크가 신호를 보내는 예시
    tokio::spawn(async move {
        for i in 1..=30 {
            tokio::time::sleep(Duration::from_secs(2)).await;
            if signal_tx.send(format!("main 채널 신호 #{i}")).await.is_err() {
                break;
            }
        }
    });

    run_client(signal_rx)
        .await
        .expect("웹 클라이언트 실행 실패.");
}

async fn run_client(
    mut signal_rx: mpsc::Receiver<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = "ws://127.0.0.1:9999";
    let (mut ws, _resp) = connect_async(url).await?;
    println!("연결됨: {}", url);
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
    let mut ping_interval = interval(Duration::from_secs(15));
    ping_interval.tick().await;
    println!("문자열 입력 후 Enter 로 전송. 종료: quit / exit (또는 Ctrl+D)");
    println!("────────────────────────────────────────");
    loop {
        line.clear();
        tokio::select! {
            biased; // 브로드캐스트 채널 이벤트에 대한 처리 우선순위를 더 늘리기 위한 선언.

            // 브로드캐스트 채널이 보낸 메시지 구독
            sig = signal_rx.recv() => {
                match sig {
                    Some(s) => println!("[신호] {s}"),
                    None => {
                        println!("(신호 채널 종료)");
                        break; // 채널로부터 채널 종료 신호가 오면 클라이언트는 모든 처리를 중지하고 종료한다.
                    }
                }
            }

            // 웹소켓에서 얻은 메시지 구독
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(t))) => println!("[서버] {}", t),
                    Some(Ok(Message::Close(f))) => {
                        println!("[서버] 연결 종료: {:?}", f);
                        return Ok(());
                    }
                    Some(Ok(other)) => println!("[서버] {:?}", other),
                    Some(Err(e)) => { // 서버가 커넥션을 강제로 끊어 버린 경우. (즉,CLOSE 를 보내지 않고 종료한 경우 임.)
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
