use walkdir::WalkDir;

// 일단 핵심 기능인 파일 탐색 기능부터 만들고, 테스트 가능하게 하기
// core 로직이고, slint 의존 없어야 함.

// 여러 참고 프로젝트 봐서 구현하기, 처음에 간단하게 하고, 점점 분리하는 식으로
// 일단 결합도 높지만 돌아가는 코드를 만들기

#[test]
fn run_walkdir_test() {
    let path = "../"; // 테스트용 경로
    let mut total_size: u64 = 0;
    let mut file_count: u64 = 0;

    for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
                file_count += 1;
            }
        }

        // 간단한 진행 표시 (일정 간격마다)
        if file_count % 1000 == 0 {
            println!("Processed {} files... total size: {} MB", file_count, total_size / 1_000_000);
        }
    }

    println!("Done! {} files, total size: {} MB", file_count, total_size / 1_000_000);
}

// 일단 트리 형태로 depth 높여가며 읽어야 하는건 맞는 듯?
//
