/// 스레드 동시성 제어 3
// 1. 특정 공통 자원에 대한 읽기 쓰레드만 존재. (별도의 Lock 등의 기법 필요 없음.)
// 2. 특정 공통 자원 하나에 race condition 을 허용하는 구조의 write 스레드 구조
// 3. 특정 공통 자원 대한 RwLock 을 통한 2.의 케이스 해소
// 4. hazard pointer 를 이용한 락 프리 모델에서의 2.의 케이스 해소
use std::hint::black_box;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use clap::{Parser, Subcommand};

struct Element {
    data: i32,
}

// --- 데모: 쓰기만, 슬립 없이 10000 도달 검증 --------------------------------
fn demo_mutex() {
    let d: Arc<Mutex<Element>> = Arc::new(Mutex::new(Element { data: 0 }));
    let mut handles = vec![];

    for _ in 0..10 {
        let data_ptr = d.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                data_ptr.lock().unwrap().data += 1;
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    println!(
        "[demo] Mutex — expected 10000, result = {}",
        d.lock().unwrap().data
    );
}

fn demo_rwlock() {
    let d: Arc<RwLock<Element>> = Arc::new(RwLock::new(Element { data: 0 }));
    let mut handles = vec![];

    for _ in 0..10 {
        let data_ptr = d.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                data_ptr.write().unwrap().data += 1;
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    println!(
        "[demo] RwLock — expected 10000, result = {}",
        d.read().unwrap().data
    );
}

// --- 벤치마크: 읽기/쓰기 비율 시나리오 ----------------------------------------

struct BenchConfig {
    name: &'static str,
    num_readers: usize,
    num_writers: usize,
    /// 스레드당 바깥 읽기 루프 횟수 (안쪽은 `read_inner_loops`와 곱해짐)
    reads_per_reader: usize,
    /// 스레드당 쓰기 횟수 (합계 쓰기 = num_writers * writes_per_writer)
    writes_per_writer: usize,
    /// 락을 잡은 채 같은 필드를 몇 번 더 읽을지. 크면 RwLock(읽기 병렬) 유리, 작으면 쓰기 경합 시 Mutex가 나을 수 있음.
    read_inner_loops: usize,
}

/// 기본: **락을 잡은 채** 같은 필드를 여러 번 읽는 추가 연산 (RwLock 유리 후보용).
/// - `Mutex`: 읽기 스레드가 이 구간에서 **직렬**로만 진행.
/// - `RwLock`: 읽기 락은 **동시에 여러 스레드**가 가질 수 있어, 이 구간이 **CPU 병렬**로 겹칠 수 있음.
const READ_INNER_LOOPS_DEFAULT: usize = 512;

fn read_work_while_locked<R>(read_guard: R, inner: usize)
where
    R: std::ops::Deref<Target = Element>,
{
    let mut acc = 0i32;
    for _ in 0..inner {
        acc = acc.wrapping_add(black_box(read_guard.data));
    }
    black_box(acc);
}

fn bench_mutex(cfg: &BenchConfig) -> (Duration, i32) {
    let data = Arc::new(Mutex::new(Element { data: 0 }));
    let start = Instant::now();
    let mut handles = vec![];

    for _ in 0..cfg.num_readers {
        let d = data.clone();
        let n = cfg.reads_per_reader;
        let inner = cfg.read_inner_loops;
        handles.push(thread::spawn(move || {
            for _ in 0..n {
                let g = d.lock().unwrap();
                read_work_while_locked(g, inner);
            }
        }));
    }
    for _ in 0..cfg.num_writers {
        let d = data.clone();
        let n = cfg.writes_per_writer;
        handles.push(thread::spawn(move || {
            for _ in 0..n {
                d.lock().unwrap().data += 1;
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    let elapsed = start.elapsed();
    let final_val = data.lock().unwrap().data;
    (elapsed, final_val)
}

fn bench_rwlock(cfg: &BenchConfig) -> (Duration, i32) {
    let data = Arc::new(RwLock::new(Element { data: 0 }));
    let start = Instant::now();
    let mut handles = vec![];

    for _ in 0..cfg.num_readers {
        let d = data.clone();
        let n = cfg.reads_per_reader;
        let inner = cfg.read_inner_loops;
        handles.push(thread::spawn(move || {
            for _ in 0..n {
                let g = d.read().unwrap();
                read_work_while_locked(g, inner);
            }
        }));
    }
    for _ in 0..cfg.num_writers {
        let d = data.clone();
        let n = cfg.writes_per_writer;
        handles.push(thread::spawn(move || {
            for _ in 0..n {
                d.write().unwrap().data += 1;
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    let elapsed = start.elapsed();
    let final_val = data.read().unwrap().data;
    (elapsed, final_val)
}

/// 시나리오당 한 번의 벤치에서 도는 **바깥** 읽기 루프 횟수 (실제 읽기 부하는 × READ_INNER_LOOPS).
/// 안쪽 루프를 두어 RwLock이 읽기 병렬로 이길 여지를 만든다 (너무 크면 벤치만 길어짐).
const TOTAL_READS_PER_RUN: usize = 1_200_000;
const TOTAL_WRITES_PER_RUN: i32 = 20_000;

/// 같은 시나리오에서 Mutex/RwLock 각각 몇 번 재측정할지 (노이즈 완화).
const BENCH_ROUNDS: usize = 7;

/// 첫 1회는 캐시·스케줄러 워밍업용으로 측정에서 제외.
const WARMUP_ROUNDS: usize = 1;

/// 총 읽기·쓰기 횟수를 시나리오 간에 맞춤 (비교 가능하도록).
fn scenarios() -> [BenchConfig; 4] {
    let total_reads = TOTAL_READS_PER_RUN;
    let total_writes = TOTAL_WRITES_PER_RUN as usize;

    [
        // 쓰기 스레드 0: Mutex는 읽기만 전부 직렬, RwLock은 읽기 병렬 → 구조적으로 RwLock 유리
        BenchConfig {
            name: "READ_ONLY + fat_read (RwLock이 이기기 쉬운 구조)",
            num_readers: 32,
            num_writers: 0,
            reads_per_reader: total_reads / 32,
            writes_per_writer: 0,
            read_inner_loops: 1024,
        },
        // 읽기 스레드 수↑, 쓰기 스레드↓ + 락 안쪽 READ_INNER_LOOPS → RwLock 유리해지기 쉬움
        BenchConfig {
            name: "read_heavy + fat_read (RwLock 유리 후보)",
            num_readers: 32,
            num_writers: 2,
            reads_per_reader: total_reads / 32,
            writes_per_writer: total_writes / 2,
            read_inner_loops: READ_INNER_LOOPS_DEFAULT,
        },
        BenchConfig {
            name: "balanced + fat_read",
            num_readers: 8,
            num_writers: 8,
            reads_per_reader: total_reads / 8,
            writes_per_writer: total_writes / 8,
            read_inner_loops: READ_INNER_LOOPS_DEFAULT,
        },
        BenchConfig {
            name: "write_heavy + thin_read (Mutex 유리 후보)",
            num_readers: 2,
            num_writers: 16,
            reads_per_reader: total_reads / 2,
            writes_per_writer: total_writes / 16,
            read_inner_loops: 1,
        },
    ]
}

fn duration_avg(samples: &[Duration]) -> Duration {
    let n = samples.len() as u128;
    let sum: u128 = samples.iter().map(|d| d.as_nanos()).sum();
    Duration::from_nanos((sum / n) as u64)
}

fn duration_median(samples: &mut [Duration]) -> Duration {
    samples.sort();
    samples[samples.len() / 2]
}

fn duration_min(samples: &[Duration]) -> Duration {
    *samples.iter().min().unwrap()
}

fn run_benchmarks() {
    println!("벤치마크: Mutex vs RwLock");
    println!(
        "① READ_ONLY: 쓰기 스레드 0 — Mutex는 모든 읽기가 직렬, RwLock은 읽기만 병렬 → 같은 총 읽기 작업에서 RwLock이 wall-clock에서 유리하기 쉬움"
    );
    println!(
        "fat_read 시나리오: 락 안 `read_inner_loops` 기본 {} — RwLock은 읽기 병렬로 이 구간이 겹칠 수 있음",
        READ_INNER_LOOPS_DEFAULT
    );
    println!("write_heavy 시나리오: read_inner_loops = 1 (얇은 읽기) — 쓰기 경합이 두드러질 때 Mutex가 나을 수 있음");
    println!(
        "공통(1회 실행당): 바깥 읽기 루프 총 {}회 (① 제외 시 나머지는 총 쓰기 {}회 동일)",
        TOTAL_READS_PER_RUN, TOTAL_WRITES_PER_RUN
    );
    println!(
        "반복: 시나리오마다 워밍업 {}회 제외 후, Mutex/RwLock 각각 {}회 측정 → min / median / avg 보고",
        WARMUP_ROUNDS, BENCH_ROUNDS
    );
    println!("측정: 전체 스레드 join 완료까지의 wall-clock 시간");
    println!("참고: RwLock이 항상 더 빠른 것은 아님 — OS/구현 오버헤드, 쓰기와의 교차, 짧은 임계구역 등에 따라 달라짐.");
    println!("권장: cargo run --release --bin thread_concurrency_3 -- bench\n");

    for cfg in scenarios() {
        let total_writes_expected = cfg.num_writers * cfg.writes_per_writer;

        for _ in 0..WARMUP_ROUNDS {
            let _ = bench_mutex(&cfg);
            let _ = bench_rwlock(&cfg);
        }

        let mut mutex_samples: Vec<Duration> = Vec::with_capacity(BENCH_ROUNDS);
        let mut rwlock_samples: Vec<Duration> = Vec::with_capacity(BENCH_ROUNDS);
        let mut last_mutex_val = 0i32;
        let mut last_rwlock_val = 0i32;

        for _ in 0..BENCH_ROUNDS {
            let (t_m, v_m) = bench_mutex(&cfg);
            let (t_r, v_r) = bench_rwlock(&cfg);
            mutex_samples.push(t_m);
            rwlock_samples.push(t_r);
            last_mutex_val = v_m;
            last_rwlock_val = v_r;
        }

        let min_m = duration_min(&mutex_samples);
        let min_r = duration_min(&rwlock_samples);
        let med_m = duration_median(&mut mutex_samples.clone());
        let med_r = duration_median(&mut rwlock_samples.clone());
        let avg_m = duration_avg(&mutex_samples);
        let avg_r = duration_avg(&rwlock_samples);

        println!("── {} ──", cfg.name);
        println!(
            "  Mutex  min/median/avg: {:>8?} / {:>8?} / {:>8?}  final_data={} (쓰기 합계 기대 {})",
            min_m, med_m, avg_m, last_mutex_val, total_writes_expected
        );
        println!(
            "  RwLock min/median/avg: {:>8?} / {:>8?} / {:>8?}  final_data={} (쓰기 합계 기대 {})",
            min_r, med_r, avg_r, last_rwlock_val, total_writes_expected
        );

        let faster = |mutex_t: Duration, rwlock_t: Duration| -> &'static str {
            if mutex_t < rwlock_t {
                "Mutex"
            } else if rwlock_t < mutex_t {
                "RwLock"
            } else {
                "동일"
            }
        };
        println!(
            "  → 더 빠른 쪽 — median: {} | min: {}",
            faster(med_m, med_r),
            faster(min_m, min_r)
        );

        println!();
    }
}

// --- CLI --------------------------------------------------------------------

#[derive(Parser)]
#[command(about = "Mutex / RwLock 데모 및 읽기·쓰기 벤치마크")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 쓰기 스레드만 10×1000 — 정합성 10000 확인 (슬립 없음)
    Demo,
    /// 읽기/쓰기 비율 3종 × Mutex vs RwLock 시간 비교
    Bench,
}

fn main() {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Commands::Bench) {
        Commands::Demo => {
            demo_mutex();
            demo_rwlock();
        }
        Commands::Bench => run_benchmarks(),
    }
}
