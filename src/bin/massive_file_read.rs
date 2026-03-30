// 대용량 파일 입출력.
// 단순히 읽어서 stdout 에 출력함. 명령어 인자로 읽을 파라미터를 받을 수도 있음. 만약 없는 경우에는 기본 값인 ./massive_file.txt 읽음.
// cargo run --bin massive_file_read {읽을 파일}

use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Parser, Debug)]
#[command(about = "읽을 파일명")]
struct CustomFile {
    file_name: Option<String>,
}

/// 파일의 각 라인을 읽어서 출력하는 함수 (전체를 다 읽지 않고, 5만 라인까지만 읽고 끝냄)
fn read_file_per_line(file_path: &String) -> io::Result<String> {

    // 파일 읽기. (해당 파일 경로는 단순 읽기만 하므로, ref 를 가져옴.)
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);

    let mut line: usize = 0; // line num 추가.
    let mut buf: String = String::new(); // 각 line 을 읽을 버퍼
    loop {

        buf.clear();

        // read_line 의 반환 타입 = Result<usize>
        let size = (reader).read_line(&mut buf)?;

        if line >= 50_000 || size == 0 {
            break;
        }

        println!("line={}, buf={}, size={}", line, buf, size);

        line += 1;
    }

    Ok(line.to_string())
}

fn main() {
    let cli = CustomFile::parse();

    let file_path = cli.file_name
        .unwrap_or_else(|| String::from("./massive_file.txt"));

    println!("Read file... {}", file_path);

    // 버전 1 = 단순 파일 줄 단위 입출력
    read_file_per_line(&file_path).unwrap();
}