use std::net::SocketAddr;
use std::time::Duration;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::{Message};

/// 웹소켓 server side

#[derive(Serialize, Deserialize)]
struct WelcomeMessage {
    msg: String
}

#[tokio::main]
async fn main() {

    run_sever().await.expect("웹소켓 서버 실행 실패.");
}

pub async fn run_sever() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    // 웹소켓 서버 실행
    let addr = "127.0.0.1:9999";
    let listener = TcpListener::bind(&addr).await?;

    println!("Listening on: {}", addr);

    // 커넥션 요청이 올 때까지 대기
    while let Ok((stream, addr)) = listener.accept().await {
        println!("Accepted connection from: {}", stream.peer_addr()?);
        tokio::spawn(handle_connection(stream, addr)); // 커넥션이 새롭게 오는 경우, task를 생성하여 TCP 연결 처리.
    }

    Ok(())
}

/// 웹소켓 서버 최초 연결 처리 수행.
pub async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    // 웹소켓 연결 처리
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            println!("❌ WebSocket 핸드셰이크 실패 {}: {}", addr, e);
            return Err(e.into());
        }
    };

    // 웹소켓 전송자, 수신자 정의
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let welcome_msg = WelcomeMessage {
        msg: "Hello, World!".to_string()
    };

    // 클라이언트에게 첫 메시지 전송 — `?`는 이 함수가 `Result`를 반환할 때만 사용 가능.
    let json = serde_json::to_string(&welcome_msg)?;
    ws_sender.send(Message::Text(json)).await?;

    /// 30초 동안 아무 수신도 없으면 유휴 타임아웃 (다음 `next()`까지 대기 시간)
    const IDLE_TIMEOUT: Duration = Duration::from_secs(30);

    loop {
        match timeout(IDLE_TIMEOUT, ws_receiver.next()).await {
            Ok(Some(Ok(Message::Text(t)))) => {
                println!("텍스트 메시지 수신 {}", t);
                ws_sender
                    .send(Message::Text("응답을 받았음.".into()))
                    .await?;
            }
            Ok(Some(Ok(Message::Binary(b)))) => println!("바이너리 수신 {:?}", b),
            Ok(Some(Ok(Message::Close(_)))) => {
                println!("클라이언트의 Close 수신");
                break;
            }
            Ok(Some(Ok(Message::Ping(payload)))) => {
                println!("클라이언트의 Ping 수신");
                ws_sender.send(Message::Pong(payload)).await?;
            }
            Ok(Some(Ok(Message::Pong(_)))) => {
                println!("클라이언트의 Pong 수신"); // 근데 이게 올 케이스는 없음.
            }
            Ok(Some(Ok(other))) => {
                println!("얜 뭘까. {}", other);
                break;
            }
            Ok(Some(Err(e))) => {
                println!("error occur! {:?}", e);
                break;
            }
            Ok(None) => break,
            Err(_) => {
                println!(
                    "❌ {} — {}초 동안 수신 없음 (유휴 타임아웃)",
                    addr,
                    IDLE_TIMEOUT.as_secs()
                );
                let _ = ws_sender.send(Message::Close(None)).await;
                break;
            }
        }
    }

    Ok(())
}
