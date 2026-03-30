// 벡터의 값을 읽는 get 메서드를 이용해서 어떤 벡터와 인덱스를 주면,
// 해당 인덱스에 있는 벡터의 원소값을 리턴하는 함수를 작성하시오.
// 만약 해당 인덱스에 값이 없으면 첫 번째 있는 원소를 리턴하고, 첫 번째 원소도 없다면 0을 리턴하시오.

fn main(){
    let v = vec![1,2,3];
    assert_eq!(1, get_val(&v, 3));
    assert_eq!(3, get_val(&v, 2));

    let v:Vec<i32> = Vec::new();
    assert_eq!(0, get_val(&v, 1));
}

// v -> Vec 에 대한 소유권을 빌림.
fn get_val(v:&Vec<i32>, idx:usize) -> i32 {

    // 1. 클로저를 써서, 간결하게 만드는 방법
    // *v.get(idx).unwrap_or_else(|| {
    //     if v.get(0).is_some() {&v[0]} else {&0}
    // })

    // 2. 클로저를 단축
    *v.get(idx).unwrap_or_else(|| &v[0])

    // 3. Vec의 get() 메서드의 반환 타입은 Option 이므로, 아래와 같이 match 로 체크
    // if v.is_empty() {
    //     return 0;
    // }
    //
    // match v.get(idx) {
    //     Some(x) => *x, // 원소에 데이터가 있을 때는 해당 원소 값 반환 (값이니, 포인터가 맞음.)
    //     None => *v.get(0).unwrap() // 아무런 원소 못 찾았을 때는 0번째 인덱스 반환
    // }

    //2.  아래는 값을 찾지 못했을 때의 패닉
    // *v.get(idx).expect(&format!("Index {} out of bounds!", idx))

    // 3. 아래는 그냥 값을 찾지 못 했을 때 패닉을 일으키는 방식
    // *v.get(idx).clone().unwrap()
}