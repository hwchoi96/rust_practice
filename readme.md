# 학습

### 1. trait object (`trait_object_practice.rs`)
- trait 는 자바의 인터페이스와 같이 하나의 공통된 것으로 묶는 것
- trait object 를 쓰면, Vec<T> 와 같은 자료형에서 하나의 trait 를 써서 서로 다른 자료형도 한 Vec에 정의할 수 있음.
- 예시) trait Addable {..} ... let v: Vec<Box<dyn Addable>> = Vec::new();
- trait_object_practice.rs

### 2. vector_basic (`vector_basic.rs`)

- `Vec::get(idx)`는 인덱스가 범위 안이면 `Some(&T)`, 아니면 `None`을 돌려준다. 인덱스로 접근할 때 `[]`는 범위 밖이면 패닉이 나지만, `get`은 패닉 없이 `Option`으로 처리할 수 있다.
- 연습 목표: 벡터와 인덱스를 받아, 해당 인덱스에 값이 있으면 그 값을, 없으면 **0번 원소**, 벡터가 비어 있으면 **0**을 반환하는 `get_val`을 만든다.
- `unwrap_or_else`로 `None`일 때 대체 참조를 고르거나, 주석에 있는 것처럼 `match`로 `Some` / `None`을 나누어 처리할 수 있다. 빈 벡터에서는 `v[0]`에 접근하면 안 되므로, 스펙대로라면 `is_empty()`나 `v.get(0)`으로 먼저 분기하는 쪽이 맞다.
- 파일 안 주석: 클로저로 `unwrap_or_else`를 쓰는 방법, `match` 버전, `expect` / `unwrap`으로 패닉 나는 경우 등을 비교해 둠.
- 실행: `cargo run --bin vector_basic`

### 3. 대용량 파일 읽기 (`massive_file_read.rs`)

대용량 텍스트 파일을 여러 방식으로 읽어 보고, 메모리·속도 관점에서 차이를 비교하는 연습이다.

**데이터**

- 약 **100만 건**을 파일에 저장한다.
- 형식은 **한 줄에 숫자 하나** (예: `1`, `2`, `3`, `4`, …)처럼 단순한 텍스트로 둔다.
- 루트의 **`massive_file.txt`** 등을 입력으로 쓴다.

**참고 (줄 읽기와 스트리밍)**  
러스트에서 “한 줄씩 읽기”의 보통 구현은 `BufReader` + `read_line` / `lines()` 이고, 이 자체가 파일 전체를 한 번에 메모리에 올리지 않는 **스트리밍**에 해당한다. 그래서 “단순 줄 읽기”와 “스트리밍”을 단계로 나누지 않고, 아래처럼 **스트리밍 vs mmap** 두 축으로 비교한다.

**진행 단계 (비교 목적)**

1. **스트리밍 (`BufReader` 등)**  
   내부 버퍼를 두고 순차적으로 읽는다. 기본 버퍼는 8KiB이며, `BufReader::with_capacity`로 크기를 키우는 실험도 할 수 있다. EOF(`read_line`이 0바이트 반환) 처리, `stdout` 잠금 등 지연·처리량에 영향 가는 부분도 함께 본다.

2. **`mmap` 방식**  
   OS의 메모리 매핑으로 파일 내용을 가상 주소 공간에 올려 두고, 필요한 구간에 접근한다. 순차 스트림과는 다른 특성(랜덤 접근, 커널 페이지 캐시 활용 등)을 비교해 본다. 구현 시에는 보통 `memmap2` 같은 크레이트를 쓴다. (텍스트 줄 단위로 쓰려면 바이트 슬라이스에서 줄 경계를 나누는 코드가 추가로 필요하다.)

**실행**

- `cargo run --bin massive_file_read`  
- 필요 시 인자로 읽을 파일 경로를 넘긴다. 생략 시 기본값은 `./massive_file.txt` 등 코드 기준.

### 4. 스레드 동시성 (`thread_concurrency_1.rs`)

여러 OS 스레드가 **같은 공통 자원**(`Arc`로 공유하는 `Vec` 등)에 접근할 때, **읽기만 / 쓰기 포함 / 락 / 락프리**를 단계적으로 비교하는 연습이다. (차익거래 봇 등 **저지연·동시성**을 염두에 둔 맥락에서 설계.)

**공통 설정 (코드 기준)**

- 공용 데이터는 예: `Element { value: String }` 를 담은 `Vec` 를 **`Arc<Vec<...>>`** 로 두고, 전역에서 한 번만 쓰고 싶을 때 **`OnceLock<Arc<Vec<...>>>`** 로 초기화하는 패턴을 시험해 볼 수 있다. (실무에서는 `main`에서 `Arc`만 만들어 넘기는 방식도 많다.)

**진행 단계 (소스 주석과 동일)**

1. **읽기 스레드만**  
   여러 스레드가 무작위 인덱스로 **조회만** 한다. 내용이 불변이면 **별도 `Mutex`/`RwLock` 없이** `Arc` 공유만으로 안전하다.

2. **쓰기 스레드 추가 (레이스 허용)**  
   같은 자원에 **읽기 + 쓰기**가 얽히도록 두어, 동기화 없이 **데이터 레이스·정합성 붕괴**가 어떻게 드러나는지 본다. (의도적으로 잘못된 예)

3. **`RwLock` 등으로 동기화**  
   2번 케이스를 **락 기반**으로 바로잡아, **락 모델에서의 정합성**을 맞춘다.

4. **락프리 + hazard pointer (고급)**  
   같은 문제를 **락 없이** 다루는 쪽과, **hazard pointer**로 **안전한 메모리 회수**를 맞추는 쪽을 비교한다. (자료구조·접근 패턴은 2·3과 완전 동일하지 않을 수 있음 — 포인터 기반 락프리 구조에서 의미가 드러나기 쉽다.)

**실행**

- `cargo run --bin thread_concurrency_1`

**`thread_concurrency_3.rs` (Mutex / RwLock 데모·벤치)**

**실행**

- `cargo run --bin thread_concurrency_3 -- demo` — 쓰기 스레드만 10×1000회, 합계 **10000** 정합성 확인 (슬립 없음).
- `cargo run --release --bin thread_concurrency_3 -- bench` — 아래 시나리오 순으로 벤치마크 (**release 권장**).

**벤치가 하는 일**

- 공유 자원 `Element { data: i32 }`에 대해 `Arc<Mutex<_>>` vs `Arc<RwLock<_>>` 로 **같은 패턴의 작업**을 수행하고, **전체 스레드 `join`까지 걸린 wall-clock**을 잰다.
- **fat_read**: 락을 잡은 뒤 `data`를 `read_inner_loops`번 더 읽는다 (`std::hint::black_box`로 최적화 방지). 이 구간에서 `Mutex`는 읽기가 **직렬**이고, `RwLock`은 읽기 락을 **동시에** 잡을 수 있어 병렬에 가깝게 동작할 수 있다.
- 시나리오마다 워밍업 1회 후, Mutex / RwLock 각각 **7회** 측정해 **min / median / avg**를 출력한다.

**시나리오 (코드 `scenarios()`와 동일)**

| 순서 | 이름 | 요지 |
|------|------|------|
| ① | `READ_ONLY + fat_read` | 쓰기 스레드 **0**. 읽기만 많고 `read_inner_loops`는 **1024**. Mutex는 읽기만 전부 직렬, RwLock은 읽기만 병렬 → **RwLock이 구조상 가장 유리하기 쉬움**. |
| ② | `read_heavy + fat_read` | 읽기 스레드 많음, 쓰기 적음. `read_inner_loops` 기본(512). |
| ③ | `balanced + fat_read` | 읽기·쓰기 스레드 수 균형. fat_read. |
| ④ | `write_heavy + thin_read` | 읽기 적음, 쓰기 많음. **`read_inner_loops = 1`** (얇은 읽기)로 쓰기 경합이 두드러지게 → **`Mutex`가 유리하기 쉬운** 후보. |

①~③은 바깥 **읽기 루프 총합**과(①은 쓰기 0) ②③④는 **총 쓰기 횟수**를 맞추도록 상수가 잡혀 있다 (`TOTAL_READS_PER_RUN`, `TOTAL_WRITES_PER_RUN` 등).

**출력 해석 (대표 패턴)**

- **①~③**: 같은 총 읽기·쓰기 부하에서 **median 기준 RwLock이 더 빠르게** 나오는 경우가 많다. (읽기 병렬 + fat 구간)
- **④**: **median 기준 Mutex가 더 빠르게** 나오는 경우가 많다. (락 획득이 가볍고 쓰기가 잦을 때 RwLock 오버헤드만 커지기 쉬움)
- **절대 시간**(초·ms)은 CPU·코어 수·OS·전원 설정에 따라 크게 달라진다. **같은 기기에서 Mutex vs RwLock 상대 비교**가 목적이다.

**예시 (한 환경에서 측정한 결과 — 숫자는 참고용)**

아래는 특정 머신에서 `--release`로 돌렸을 때의 **대략적인 비율**이다. 재현 시 위·아래 순서와 승자 패턴만 비슷하면 된다.

- **① READ_ONLY + fat_read**: Mutex 전체 시간이 RwLock보다 **수 배 ~ 한 자릿수 배** 길게 나오는 경우가 많다.
- **② read_heavy / ③ balanced**: Mutex가 **수 초대**, RwLock이 **1초 미만~1초대**처럼 **RwLock이 짧게** 나오는 패턴이 흔하다.
- **④ write_heavy + thin_read**: Mutex가 **수십 ms**, RwLock이 **100ms 안팎**처럼 **Mutex가 짧게** 나오는 패턴이 흔하다.

출력 마지막 줄의 `→ 더 빠른 쪽 — median: … | min: …` 로 시나리오별 승자를 바로 볼 수 있다.

아래는 벤치마크 테스트 결과.

```
── READ_ONLY + fat_read (RwLock이 이기기 쉬운 구조) ──
Mutex  min/median/avg: 9.445657917s / 9.564503417s / 9.582837952s  final_data=0 (쓰기 합계 기대 0)
RwLock min/median/avg: 1.606396542s / 1.621629125s / 1.634122821s  final_data=0 (쓰기 합계 기대 0)
→ 더 빠른 쪽 — median: RwLock | min: RwLock

── read_heavy + fat_read (RwLock 유리 후보) ──
Mutex  min/median/avg: 5.337699042s / 5.353763084s / 5.367723446s  final_data=20000 (쓰기 합계 기대 20000)
RwLock min/median/avg: 905.767958ms / 919.426334ms / 921.276428ms  final_data=20000 (쓰기 합계 기대 20000)
→ 더 빠른 쪽 — median: RwLock | min: RwLock

── balanced + fat_read ──
Mutex  min/median/avg: 5.342283042s / 5.365264209s / 5.373380577s  final_data=20000 (쓰기 합계 기대 20000)
RwLock min/median/avg: 903.47675ms / 937.441958ms / 940.714231ms  final_data=20000 (쓰기 합계 기대 20000)
→ 더 빠른 쪽 — median: RwLock | min: RwLock

── write_heavy + thin_read (Mutex 유리 후보) ──
Mutex  min/median/avg: 67.21075ms / 76.205375ms / 75.366708ms  final_data=20000 (쓰기 합계 기대 20000)
RwLock min/median/avg: 102.469833ms / 111.72725ms / 110.271047ms  final_data=20000 (쓰기 합계 기대 20000)
→ 더 빠른 쪽 — median: Mutex | min: Mutex
```
