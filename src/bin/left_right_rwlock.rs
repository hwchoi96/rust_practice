use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

/// left-right 기반 데이터 교체 방법 1
/// > 쓰기 행위에는 비활성화된 인덱스에 배타락을 걸어서 안전하게 교체.

struct SimpleLeftRight<T> {

    active_index: AtomicUsize,
    data: Arc<[RwLock<T>; 2]> // left-right 데이터 생성.
}

impl<T: Clone> SimpleLeftRight<T> {

    /// left-right 초기 데이터 생성.
    fn new(init: T) -> SimpleLeftRight<T> {

        Self {
            active_index: AtomicUsize::new(0),
            data: Arc::new([RwLock::new(init.clone()), RwLock::new(init)])
        }
    }

    /// left-right 데이터 조회.
    fn read(&self) -> T where T: Clone {

        // 자료구조의 active 된 인덱스를 읽어서, 복사본을 전달한다.
        let idx = self.active_index.load(Ordering::Acquire);

        self.data[idx].read().unwrap().clone()
    }

    /// left-right 데이터 쓰기.
    fn write(&self, value: T) {

        let active = self.active_index.load(Ordering::Acquire);
        // 쓰기를 활성화할 인덱스를 계산한다.
        let inactive = 1 - active;

        // 비활성화된 인덱스에 RwLock 를 통해, write.
        *self.data[inactive].write().unwrap() = value;

        // 쓰기 행위가 완료되고, 활성화 인덱스를 방금 교체한 인덱스로 변경함.
        self.active_index.store(inactive, Ordering::Release);
    }
}

fn main() {
    println!("=== SimpleLeftRight (RwLock 슬롯) 테스트 ===\n");

    // 1) 단일 스레드
    let lr = Arc::new(SimpleLeftRight::new("초기값".to_string()));
    assert_eq!(lr.read(), "초기값");
    lr.write("첫 갱신".to_string());
    assert_eq!(lr.read(), "첫 갱신");
    println!("1) 단일 스레드: 초기값 → 첫 갱신 확인 OK\n");

    // 2) 읽기 여러 개 + 쓰기 하나
    let lr = Arc::new(SimpleLeftRight::new("공유-시작".to_string()));
    let writer_lr = Arc::clone(&lr);
    let writer = thread::spawn(move || {
        for i in 0..8 {
            writer_lr.write(format!("버전-{i}"));
            thread::sleep(Duration::from_millis(15));
        }
    });

    let mut readers = Vec::new();
    for id in 0..3 {
        let rlr = Arc::clone(&lr);
        readers.push(thread::spawn(move || {
            for _ in 0..25 {
                let v = rlr.read();
                println!("  [reader {id}] {v}");
                thread::sleep(Duration::from_millis(8));
            }
        }));
    }

    writer.join().expect("writer panic");
    for (i, h) in readers.into_iter().enumerate() {
        h.join().unwrap_or_else(|_| panic!("reader {i} panic"));
    }

    println!("\n2) 동시 테스트 종료. 마지막 read: {:?}\n", lr.read());
    println!("전체 완료.");
}