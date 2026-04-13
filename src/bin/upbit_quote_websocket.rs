use futures_util::{SinkExt, StreamExt};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::interval;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct WebsocketQuote {
    code: String,
    opening_price: Decimal,
    high_price: Decimal,
    low_price: Decimal,
    trade_price: Decimal,
    prev_close_price: Decimal,
    change: String,
    change_price: Decimal,
    signed_change_rate: Decimal,
    trade_volume: Decimal,
    ask_bid: String,
}

const UPBIT_QUOTE_URL: &str = "wss://api.upbit.com/websocket/v1";
const PING_INTERVAL_SECS: u64 = 55;

/// https://docs.upbit.com/kr/reference/websocket-ticker
#[tokio::main]
pub async fn main() {
    let pairs = vec!["KRW-BTC", "KRW-ETH", "KRW-XRP", "KRW-USDT", "KRW-SOL"];

    get_quote(pairs).await.expect("Can't get quote");
}

async fn get_quote(pairs: Vec<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let (ws_stream, _) = connect_async(UPBIT_QUOTE_URL).await?;

    let (mut write, mut read) = ws_stream.split();

    let subscribe_info = serde_json::json!([
        { "ticket": Uuid::new_v4().to_string() },
        { "type": "ticker", "codes": pairs} ,
        { "format": "DEFAULT"}
    ]);
    let subscribe_msg = subscribe_info.to_string();

    write.send(Message::Text(subscribe_msg)).await?;

    // 핑퐁 전용 핑 interval
    let mut ping_interval = interval(Duration::from_secs(PING_INTERVAL_SECS));
    ping_interval.tick().await;

    loop {
        tokio::select! {
            next = read.next() => {
                match next {
                    // 업비트는 티커 JSON을 Text가 아니라 Binary(UTF-8)로 보내는 경우가 많음.
                    Some(Ok(Message::Text(text))) => {
                        println!("{}", text);
                    }
                    Some(Ok(Message::Binary(bin))) => {
                        match String::from_utf8(bin) {
                            Ok(s) => println!("{}", s),
                            Err(e) => println!("binary (non-utf8): len={} err={}", e.as_bytes().len(), e),
                        }
                    }
                    Some(Ok(other)) => {
                        println!("기타 프레임: {:?}", other);
                    }
                    Some(Err(e)) => {
                        println!("error: {:?}", e);
                    }
                    None => {
                        println!("스트림 종료");
                        return Ok(());
                    }
                }
            }
            _ = ping_interval.tick() => {
                write.send(Message::Text("PING".into())).await?;
                println!("업비트 PING 전송")
            }
        }
    }
}
