# Cursor / AI 세션용 프로젝트 요약

이 문서는 **대화 맥락이 없는 상태**에서 이 레포를 다시 열었을 때, 빠르게 같은 줄에서 작업을 이어가기 위한 메모다. 상세한 학습 노트는 `readme.md`를 본다.

## 프로젝트가 뭔지

- **이름**: `basic_practice` (Rust 개인 학습용 레포)
- **에디션**: `Cargo.toml` 기준 Rust **2024**
- **의존성**: 현재 `[dependencies]` 비어 있음 (`memmap2` 등은 `file_massive_read` 구현 시 추가 예정일 수 있음)

## 디렉터리 구조 (요지)

```
basic_practice/
├── Cargo.toml
├── Cargo.lock
├── readme.md              # 학습 주제별 정리 (trait object, vector, 대용량 파일 예정)
├── cursor.md              # 이 파일 (세션 재시작용 요약)
├── massive_file.txt       # 대용량 읽기 연습용 데이터 (루트에 둠)
├── src/
│   ├── main.rs            # 기본 바이너리 엔트리 (거의 비어 있음)
│   └── bin/
│       ├── trait_object_practice.rs
│       └── vector_basic.rs
└── target/                # 빌드 산출물 (.gitignore)
```

- **여러 실행 파일**: `src/bin/*.rs` → 각각 `cargo run --bin <파일명_확장자_제외>` 로 실행.

## 이미 있는 바이너리

| 바이너리 | 파일 | 한 줄 설명 |
|----------|------|------------|
| `basic_practice` | `src/main.rs` | 기본 `main`, 비어 있음에 가깝다. |
| `trait_object_practice` | `src/bin/trait_object_practice.rs` | `Vec<Box<dyn Addable>>` 등 trait object 실습. |
| `vector_basic` | `src/bin/vector_basic.rs` | `Vec::get`, `get_val` 연습 (주석에 여러 구현 시도). |

## 대용량 파일 읽기 (진행 예정)

- **목표 파일 (코드)**: `src/bin/file_massive_read.rs` — **아직 없을 수 있음**. 추가되면 `cargo run --bin file_massive_read`.
- **의도한 비교** (readme §3과 동일):
  1. 줄 단위로 단순 읽기·출력
  2. 스트리밍(버퍼/이터레이터 등, 전체를 메모리에 안 올리기)
  3. `mmap` (보통 `memmap2` 크레이트)
- **입력 데이터**: 루트의 **`massive_file.txt`**
  - **약 100만 줄**, 한 줄에 정수 하나 (`1` … `1000000`).
  - 크기 대략 **6~7MB** 수준.
  - macOS `seq`는 큰 수를 과학적 표기로 쓸 수 있어, 생성 시 **`awk 'BEGIN{for(i=1;i<=1000000;i++)print i}'`** 같은 방식이 안전하다.

## 자주 쓰는 명령

```bash
cargo build
cargo run --bin vector_basic
cargo run --bin trait_object_practice
# file_massive_read 추가 후:
# cargo run --bin file_massive_read
```

## 세션에서 우선 볼 곳

1. **`readme.md`** — 주제별 상세·링크 대상 파일명.
2. **`Cargo.toml`** — 크레이트 이름·에디션·의존성.
3. **`src/bin/`** — 실제 학습 코드 위치.
4. **`massive_file.txt`** — 대용량 읽기 과제의 고정 입력으로 가정해도 된다 (경로: 레포 루트).
