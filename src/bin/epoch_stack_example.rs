//! Treiber 스택 — `crossbeam-epoch`으로 **락 없이** push/pop.
//! pop에서 제거한 노드는 `Guard::defer_destroy`로 epoch이 지난 뒤 해제된다.
//!
//! 실행: `cargo run --release --bin epoch_stack_example`

use crossbeam_epoch::{self as epoch, Atomic, Owned};
use std::sync::atomic::Ordering;

/// 스택 노드
struct Node {
    value: i32,
    next: Atomic<Node>,
}

/// 락프리 LIFO 스택
pub struct LockFreeStack {
    head: Atomic<Node>,
}

impl LockFreeStack {
    pub fn new() -> Self {
        LockFreeStack {
            head: Atomic::null(),
        }
    }

    /// 맨 앞에 삽입 (CAS 루프)
    pub fn push(&self, value: i32) {
        let guard = &epoch::pin();
        let mut n = Owned::new(Node {
            value,
            next: Atomic::null(),
        });

        loop {
            let head = self.head.load(Ordering::Acquire, guard);
            n.next.store(head, Ordering::Relaxed);
            match self.head.compare_exchange(
                head,
                n,
                Ordering::Release,
                Ordering::Relaxed,
                guard,
            ) {
                Ok(_) => return,
                Err(e) => n = e.new,
            }
        }
    }

    /// 맨 앞 제거. 제거된 노드는 다른 스레드가 아직 `Shared`로 읽을 수 있으므로 바로 free하지 않고
    /// epoch 기반 지연 해제(`defer_destroy`)에 맡긴다.
    pub fn pop(&self) -> Option<i32> {
        let guard = &epoch::pin();
        loop {
            let head = self.head.load(Ordering::Acquire, guard);
            if head.is_null() {
                return None;
            }

            let node = unsafe { head.as_ref().unwrap() };
            let next = node.next.load(Ordering::Acquire, guard);

            match self.head.compare_exchange(
                head,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
                guard,
            ) {
                Ok(_) => {
                    let v = node.value;
                    unsafe {
                        guard.defer_destroy(head);
                    }
                    return Some(v);
                }
                Err(_) => {}
            }
        }
    }
}

fn main() {
    let stack = LockFreeStack::new();
    let s = std::sync::Arc::new(stack);

    let mut handles = vec![];
    for t in 0..4 {
        let s = s.clone();
        handles.push(std::thread::spawn(move || {
            for i in 0..256 {
                s.push(t * 1000 + i);
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }

    let mut count = 0;
    while s.pop().is_some() {
        count += 1;
    }
    println!("popped {count} items (expected 1024)");
}
