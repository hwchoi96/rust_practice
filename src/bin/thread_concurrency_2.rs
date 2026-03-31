/// 스레드 동시성 제어 2
// 1. 특정 공통 자원에 대한 읽기 쓰레드만 존재. (별도의 Lock 등의 기법 필요 없음.)
// 2. 특정 공통 자원 하나에 race condition 을 허용하는 구조의 write 스레드 구조
// 3. 특정 공통 자원 대한 RwLock 을 통한 2.의 케이스 해소
// 4. hazard pointer 를 이용한 락 프리 모델에서의 2.의 케이스 해소
use std::sync::{Arc};
use std::thread;

struct Data {
    value: i32,
}

fn main() {
    // atomic wrap variable 선언.
    let arc_v: Arc<Data> = Arc::new(Data { value: 0 });

    let mut handles = vec![];

    // 10개의 스레드를 생성하여, 각 스레드마다 하나의 자원의 카운트를 1씩 총 1000번 증가.
    for _ in 0..10 {
        let data_ptr = arc_v.clone();

        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                // 의도적으로 unsafe 및 arc 필드 데이터 동시 변경 수행
                unsafe {
                    let ptr = Arc::as_ptr(&data_ptr) as *mut Data;
                    (*ptr).value += 1;
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // 실제 카운트는 10000이 아닐 수도 있음.
    println!("multi thread race condition test = 예상 카운트: 10_000, 실제 카운트: {}", arc_v.value);
}
