//! 이벤트 드리븐에 가까운 패턴: **채널**로 “쓰기(전송)”하면 **읽기(수신) 쪽만** 깨어나 처리한다.
//! 공유 `Option`을 돌려가며 폴링하는 것보다, 실무에서는 이런 **메시지 패싱**이 흔하다.
//!
//! 실행: `cargo run --bin channel_event_example`

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// 프로듀서가 보내는 이벤트 (실무에서는 명령/도메인 이벤트로 치환)
#[derive(Debug)]
enum Event {
    Tick(i32),
    Done,
}

fn main() {
    let (tx, rx) = mpsc::channel::<Event>();

    // --- “쓰기” 역할: 이벤트를 넣는 스레드(여러 개여도 됨 — clone(tx)) ---
    let producer = thread::spawn(move || {
        for i in 0..5 {
            thread::sleep(Duration::from_millis(50));
            tx.send(Event::Tick(i)).expect("receiver alive");
        }
        tx.send(Event::Done).ok();
    });

    // --- “읽기” 역할: `None`이 아닌 메시지가 올 때만 루프가 진행 (`recv`는 블로킹) ---
    // `for event in rx` 는 채널이 닫힐 때까지 이터레이터처럼 동작(스트림에 가깝게 쓰는 동기 패턴).
    for event in rx {
        match event {
            Event::Tick(n) => println!("[consumer] 처리: Tick({n})"),
            Event::Done => {
                println!("[consumer] Done 수신 → 종료");
                break;
            }
        }
    }

    producer.join().expect("producer panic");
}
