//! 락 없이 동시에 갱신하는 **원자 카운터**와, **고정 슬롯**을 쓰는 **맵에 가까운** 패턴.
//!
//! - 카운터: `AtomicUsize` — 가장 흔한 락프리 정수 갱신.
//! - 맵: 키를 `0..N`으로 제한하면 슬롯마다 `AtomicU64`를 두고 `fetch_add` / `load` 만으로
//!   뮤텍스 없이 카운트를 올릴 수 있다. (해시 충돌 없이 키=슬롯 인덱스)
//!
//! 일반적인 **가변 크기 동시 해시맵**은 `std`에 없고, `dashmap`(샤딩+락), `flurry`(RCU 계열) 등
//! 크레이트를 쓰는 경우가 많다 — 완전 락프리인지는 구현체마다 다름.
//!
//! 실행: `cargo run --bin atomic_lockfree_demo`

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

// --- 1) 원자 카운터: 여러 스레드가 동시에 += --------------------------------

fn demo_atomic_counter() {
    let total = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let t = total.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                // Relaxed: 순서가 다른 변수와 엮이지 않으면 보통 충분 (카운터 합계만 볼 때)
                t.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }

    println!(
        "[counter] AtomicUsize 합계 = {} (기대 10000)",
        total.load(Ordering::SeqCst)
    );
}

// --- 2) 고정 키 맵: 키 k ∈ 0..16 → 슬롯 k, 슬롯마다 AtomicU64 ----------------

const SLOTS: usize = 16;

struct AtomicSlotMap {
    /// key % SLOTS 가 아니라 **키 자체가 0..SLOTS** 라고 가정 → 충돌 없음 (교육용)
    slots: [AtomicU64; SLOTS],
}

impl AtomicSlotMap {
    fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| AtomicU64::new(0)),
        }
    }

    fn add(&self, key: usize, delta: u64) {
        debug_assert!(key < SLOTS);
        self.slots[key].fetch_add(delta, Ordering::Relaxed);
    }

    fn get(&self, key: usize) -> u64 {
        debug_assert!(key < SLOTS);
        self.slots[key].load(Ordering::Acquire)
    }
}

fn demo_atomic_slot_map() {
    let map = Arc::new(AtomicSlotMap::new());
    let mut handles = vec![];

    // 스레드마다 서로 다른 키에만 쓰면 충돌 없음; 같은 키에 동시 += 도 원자 연산이라 안전
    for tid in 0..4 {
        let m = map.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                let key = tid % SLOTS; // 0,1,2,3 에 반복 기록
                m.add(key, 1);
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }

    for k in 0..4 {
        println!("[slot map] key {k} count = {}", map.get(k));
    }
}

fn main() {
    demo_atomic_counter();
    demo_atomic_slot_map();
}
