# 학습

### 1. trait object 
- trait 는 자바의 인터페이스와 같이 하나의 공통된 것으로 묶는 것
- trait object 를 쓰면, Vec<T> 와 같은 자료형에서 하나의 trait 를 써서 서로 다른 자료형도 한 Vec에 정의할 수 있음.
- 예시) trait Addable {..} ... let v: Vec<Box<dyn Addable>> = Vec::new();
- trait_object_practice.rs

