use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};

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

}