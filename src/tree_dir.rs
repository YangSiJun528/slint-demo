use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug)]
struct DirNode {
    path: PathBuf,
    name: String,
    children: Vec<DirNode>,
    depth: usize,
}

impl DirNode {
    fn new(path: PathBuf, depth: usize) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("/")
            .to_string();

        Self {
            path,
            name,
            children: Vec::new(),
            depth,
        }
    }

    fn print_tree(&self) {
        let indent = "  ".repeat(self.depth);
        println!("{}├─ {}", indent, self.name);

        for child in &self.children {
            child.print_tree();
        }
    }
}

fn build_tree(root_path: &str, max_depth: usize) -> DirNode {
    let root = PathBuf::from(root_path);
    let mut root_node = DirNode::new(root.clone(), 0);

    // 깊이별로 디렉토리 수집
    let mut dirs_by_depth: Vec<Vec<PathBuf>> = vec![Vec::new(); max_depth + 1];

    for entry in WalkDir::new(&root)
        .max_depth(max_depth)
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.file_type().is_dir() {
            let depth = entry.depth();
            if depth > 0 && depth <= max_depth {
                dirs_by_depth[depth].push(entry.path().to_path_buf());
            }
        }
    }

    // 트리 구조 생성
    build_tree_recursive(&mut root_node, &dirs_by_depth, 1);

    root_node
}

fn build_tree_recursive(
    parent: &mut DirNode,
    dirs_by_depth: &[Vec<PathBuf>],
    current_depth: usize,
) {
    if current_depth >= dirs_by_depth.len() {
        return;
    }

    // 현재 노드의 직속 자식들만 찾기
    for dir_path in &dirs_by_depth[current_depth] {
        if let Some(dir_parent) = dir_path.parent() {
            if dir_parent == parent.path {
                let mut child = DirNode::new(dir_path.clone(), current_depth);
                build_tree_recursive(&mut child, dirs_by_depth, current_depth + 1);
                parent.children.push(child);
            }
        }
    }
}

#[test]
fn run_tree_test() {
    let path = "../";
    let max_depth = 3; // 탐색할 최대 깊이

    println!("Building directory tree...");
    let tree = build_tree(path, max_depth);

    println!("\nDirectory structure:");
    tree.print_tree();

    println!("\nStats:");
    let total_dirs = count_dirs(&tree);
    println!("Total directories: {}", total_dirs);
}

fn count_dirs(node: &DirNode) -> usize {
    1 + node.children.iter().map(|c| count_dirs(c)).sum::<usize>()
}

// 더 효율적인 버전: 한 번 순회하면서 직접 트리 구성
// 2분 30초 정도 걸림. 디스크 정보만 보는건데도 이정도면 좀 오래걸릴듯?
// 이거 병렬 dfs로 빠르게 메타데이터만 읽게 하고, 용량 구하는건 병렬로 나중에 처리하게 하면 될 듯?
// 초기에 느린건 점 어떻게든 잘 해결해봐야 할거 같고
#[test]
fn run_tree_test_v2() {
    let path = "/";
    let max_depth = 10000;

    println!("Building directory tree (v2 - efficient)...");
    let tree = build_tree_efficient(path, max_depth);

    println!("\nDirectory structure:");
    tree.print_tree();

    println!("\nStats:");
    println!("Total directories: {}", count_dirs(&tree));
}

fn build_tree_efficient(root_path: &str, max_depth: usize) -> DirNode {
    use std::collections::HashMap;

    let root = PathBuf::from(root_path);
    let root_node = DirNode::new(root.clone(), 0);

    // PathBuf -> DirNode 매핑
    let mut nodes: HashMap<PathBuf, DirNode> = HashMap::new();
    nodes.insert(root.clone(), root_node);

    for entry in WalkDir::new(&root)
        .max_depth(max_depth)
        .min_depth(1) // 루트는 이미 처리
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.file_type().is_dir() {
            let path = entry.path().to_path_buf();
            let depth = entry.depth();
            let node = DirNode::new(path.clone(), depth);
            nodes.insert(path, node);
        }
    }

    // 부모-자식 관계 구성
    let paths: Vec<PathBuf> = nodes.keys().cloned().collect();

    for path in paths {
        if path == root {
            continue;
        }

        if let Some(parent_path) = path.parent() {
            if let Some(node) = nodes.remove(&path) {
                if let Some(parent) = nodes.get_mut(parent_path) {
                    parent.children.push(node);
                }
            }
        }
    }

    nodes.remove(&root).unwrap()
}

// 일단 요구사항이 좀 명확하진 듯?

// Rust로 ncdu 같은 도구를 GUI, CLI 버전으로 만들고 싶다. (Slint를 사용하긴 하지만, 코어 로직은 UI와 별개로 순수해야 한다.)
//
// 차이점은 파일 매니저가 내장되어 있어서 그래프가 아니라 디렉토리별로 얼마나 차지하는지 명확하게 알 수 있어야 한다. OmniDiskSweeper나 WinDirStat 같은 서비스를 원한다.
//
// 이를 위해서 필요한 요구사항은 다음과 같다. 이를 어떻게 기술적으로 해결할 수 있을지 여러 방법으로 고민하고 알려줘.
//
// 1. 앱을 열었을 때 or 목표 폴더를 선택했을 때, 하위 요소들이 계층 형식으로 보이고, 로딩시간이 없어야 함. (단 파일 크기는 계산중일 수 있는데, 폴더/파일 요소가 누락되면 안된다.)
// 1-1. 이를 위해선 최대한 빠르게 전체 구조를 읽되, 사용자가 접근 가능할만한(지금 열어봤거나, 열어본 것들의 하위 목록) 정보를 우선하여 읽어 사용자가 딜레이를 거의 느끼지 못해야 한다.
// 2. 파일/폴더의 크기를 파일 이름 옆에 보여준다. 계산중이라면 정확하지 않을 수 있지만, 계산 후에는 정확해야 한다.
//
// 3. 앱이 열려있는 동안 사용자가 파일을 삭제하거나 추가한 경우, 그 변경에 맞게 정보를 보완한다.
//
// 4. (옵션) 필요하다면 도표 등의 형태로 보여준다. 단, 이는 1,2가 해결된다면 손쉬운 문제이므로 신경쓰지 않는다.
