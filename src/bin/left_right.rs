use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

struct LeftRight<T> {
    active_index: AtomicUsize,
    data: Arc<[AtomicPtr<T>; 2]> // crossbeam 을 통한 left right algorithm 구현.
}

impl<T> LeftRight<T>
where
    T: Clone + Send,
{
    pub fn new(element: T) -> LeftRight<T> {

        Self {
            active_index: AtomicUsize::new(0),
            data: Arc::new([
                AtomicPtr::new(Box::into_raw(Box::new(element.clone()))),
                AtomicPtr::new(Box::into_raw(Box::new(element)))
            ])
        }
    }

    pub fn read(&self) -> T {

        // 1. read 시 안전한 메모리 해지 유도를 위한 Guard 생성.
        let _guard = crossbeam_epoch::pin();

        // 2. 액티브 인덱스 계산.
        let active = self.active_index.load(Ordering::Acquire);

        // 3. 데이터 복사본 반환
        let p = self.data[active].load(Ordering::Acquire);
        assert!(!p.is_null());
        unsafe { (*p).clone() }
    }

    pub fn write(&self, element: T) {

        let active = self.active_index.load(Ordering::Acquire);
        let inactive = 1 - active; // active 는 0 또는 1만 가정

        let new_ptr = Box::into_raw(Box::new(element));
        let old_ptr = self.data[inactive].swap(new_ptr, Ordering::AcqRel);

        let g = crossbeam_epoch::pin();
        if !old_ptr.is_null() {
            // `defer` 클로저는 `Send` 필요. 제네릭 `T`일 때 `*mut T: Send` 추론이 안 되는 경우가 있어 주소만 넘김.
            let addr = old_ptr as usize;
            g.defer(move || unsafe {
                drop(Box::from_raw(addr as *mut T));
            });
        }

        self.active_index.store(inactive, Ordering::Release);
    }
}

fn main() {
    println!("=== LeftRight<String> 테스트 ===\n");

    // 1) 단일 스레드: 초기값 → write → read
    let lr = Arc::new(LeftRight::new("초기값".to_string()));
    assert_eq!(lr.read(), "초기값");
    lr.write("첫 갱신".to_string());
    assert_eq!(lr.read(), "첫 갱신");
    println!("1) 단일 스레드: 초기값 → 첫 갱신 확인 OK\n");

    // 2) 여러 읽기 스레드 + 한 쓰기 스레드
    let lr = Arc::new(LeftRight::new("공유-시작".to_string()));
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

    let last = lr.read();
    println!("\n2) 동시 테스트 종료. 마지막 read: {last:?}\n");

    println!("전체 완료.");
}