/// trait object 샘플링

// 공통 트레이트 정의
trait Addable {
    fn add(&self);
}

struct AStruct {
    amount: u32,
    name: String
}

impl Addable for AStruct {
    fn add(&self) {
        println!("{} ~ {}", self.amount, self.name);
    }
}

struct BStruct<'a> {
    string1: &'a str, // 라이프타임이 있는 변수 선언
    float64: f64
}

impl<'a> Addable for BStruct<'a> {
    fn add(&self) {
        println!("{} ! {}", self.string1, self.float64)
    }
}

struct CStruct {
    num1: i32,
    num2: u32,
}

impl Addable for CStruct {

    fn add(&self) {
        println!("{} @ {}", self.num1, self.num2)
    }
}

pub fn trait_object() {

    let mut v: Vec<Box<dyn Addable>> = Vec::new();

    v.push(Box::new(AStruct {amount: 2, name: String::from("Bob")}));
    v.push(Box::new(BStruct {string1: "Alice", float64: 3.14}));
    v.push(Box::new(CStruct {num1: 3, num2: 5}));

    for x in v {
        x.add()
    }
}

fn main() {
    trait_object();
}