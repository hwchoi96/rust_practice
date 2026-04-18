use std::sync::Arc;
/// lock-free Swap 멀티 스레드 환경에서 못 쓰는 버전.
/// Mutex 등의 임계 영역 설정이 없기 때문에, 쓰기 스레드가 데이터 변경 후 메모리 해제하면 UB 발생 가능.
/// 데이터를 안전하게 처리하려면, AtomicSwap, epoch 등 추가 구현체 사용 필요.

use std::sync::atomic::{AtomicPtr, Ordering};

struct MySwap<T> {

    ptr: AtomicPtr<T>
}

impl<T> MySwap<T> {

    /// 데이터 초기화
    pub fn new(element: T) -> MySwap<T> {

        // AtomicPtr 은 데이터의 메모리 공간만 atomic 하게 관리함.
        // 즉, 내부 데이터를 Box로 생성하여 힙 메모리에 래핑하고 그 공간을 가리키도록 구성해야함.
        let data = Box::into_raw(Box::new(element));

        Self {
            ptr: AtomicPtr::new(data)
        }
    }

    /// Swap 데이터 조회
    pub fn get(&self) -> T where T: Clone {

        let p = self.ptr.load(Ordering::Acquire);
        assert!(!p.is_null());
        // 현재 AtomicPtr 가 가리키는 데이터의 복사본을 반환
        unsafe { (*p).clone() }
    }

    /// Swap 데이터 변경
    pub fn set(&self, element: T) {

        let new_data = Box::into_raw(Box::new(element));
        // 이전에 저장되어 있던 값을 아예 바꿔치기 해버림.
        let old = self.ptr.swap(new_data, Ordering::Release);

        // 아래 코드 때문에 멀티 스레드 환경에서 UB가 발생할 수 있음.
        // 읽기 스레드가 get 에서 self.ptr.load() 를 처리한 이후, 컨텍스트 스위칭이 되어 쓰기 스레드가 아래 코드가 실행되어 구 주소의 메모리를 해제해버릴 수 있음.
        assert!(!old.is_null());
        unsafe {
            drop(Box::from_raw(old));
        }
    }
}

#[tokio::main]
async fn main() {

    let shared_data = Arc::new(MySwap::new(String::from("Hello, world!")));

    // 읽기 스레드 정의
    for _ in 0..10 {
        let sd = Arc::clone(&shared_data);
        tokio::spawn(async move {
            loop {
                let data = sd.get();

                println!("{}", data);

                // 쓰기 스레드에 의해, 읽기 쓰레드는 UB가 발생할 수 있음.
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });
    }

    let wd = Arc::clone(&shared_data);
    tokio::spawn(async move {
        loop {
            let write_data = String::from("데이터를 바꿔 버릴거임.");

            println!("쓰기 스레드에서 데이터를 변경합니다. 매우 간헐적으로 UB가 발생할 수 있습니다.");

            // 쓰기 스레드에서 데이터 변경
            wd.set(write_data);

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });

    // `main`이 여기서 바로 끝나면 런타임이 즉시 내려가 스폰 태스크가 거의 돌지 못함 → 출력이 1~몇 줄에서 끊길 수 있음.
    eprintln!("(종료: Ctrl+C)");
    tokio::signal::ctrl_c().await.ok();
}
