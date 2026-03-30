use std::sync::OnceLock;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use rand::Rng;

/// 스레드 동시성 제어 1
// 1. 특정 공통 자원에 대한 읽기 쓰레드만 존재. (별도의 Lock 등의 기법 필요 없음.)
// 2. 특정 공통 자원 하나에 race condition 을 허용하는 구조의 write 스레드 구조
// 3. 특정 공통 자원 대한 RwLock 을 통한 2.의 케이스 해소
// 4. hazard pointer 를 이용한 락 프리 모델에서의 2.의 케이스 해소

struct Element {
    value: String
}

// OnceLock 테스트용으로 읽기 전용 스레드에서 쓸 공용 자원 생성.
// > OnceLock 은 슬롯에 대한 초기화를 한번만 하는 것임.
// > 안에 들어가는 데이터가 가변 변수인 경우에는 데이터 수정이 가능함.
static READ_ONLY_SHARED_RESOURCES: OnceLock<Arc<Vec<Element>>> = OnceLock::new();
fn init() -> &'static Arc<Vec<Element>> {
    READ_ONLY_SHARED_RESOURCES.get_or_init(|| {
        Arc::new(
            vec! [
                Element { value: "a".to_string() },
                Element { value: "b".to_string() },
                Element { value: "c".to_string() },
                Element { value: "d".to_string() }
            ]
        )
    })
}

/// 랜덤 인덱스 반환
fn get_random_index() -> usize {
    rand::thread_rng().gen_range(0..READ_ONLY_SHARED_RESOURCES.get().unwrap().len())
}

/// 특정 공통 자원에 대한, 단순 조회 기능만 수행
fn case_1() {

    let mut handles = vec![];

    // 스레드 10개 생성
    for id in 0..10 {
        let t = thread::spawn(move || {

            // 한 스레드마다 10번씩 데이터를 꺼내서 출력함.
            for i in 0..10 {
                let index = get_random_index();

                let value = READ_ONLY_SHARED_RESOURCES.get().unwrap().get(index).unwrap();

                println!("thread id = {}, get value = {}", id, value.value);

                thread::sleep(Duration::from_millis(100));
            }
        });
        handles.push(t);
    }

    for h in handles {
        h.join().unwrap();
    }
}

fn main() {

    init(); // 읽기 전용 공통 자원 초기화

    case_1(); // 읽기 전용 스레드 생성 후 실행.
}