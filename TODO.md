## 핵심 아키텍처 설계

### 요구사항 분석
1번과 3번이 핵심 난제입니다. 즉각적인 디렉토리 구조 표시와 실시간 파일시스템 감시를 동시에 해결해야 합니다.

---

## 1. 디렉토리 스캔 전략 (요구사항 1, 2)

### 방법 A: 2-Phase 스캔 (권장)
**Phase 1: 구조만 빠르게**
- `read_dir()` 순회만으로 파일/폴더 이름, 타입만 수집
- 크기는 `None` 또는 `-1`로 표시
- UI에 즉시 렌더링 (< 100ms)

**Phase 2: 크기 계산**
- 백그라운드 스레드풀에서 Bottom-up 계산
- 리프 노드(파일)부터 시작해서 부모로 전파
- 계산 완료된 노드만 UI 업데이트

**장점:**
- 사용자가 즉시 폴더 구조 탐색 가능
- 대용량 디렉토리(수만 개 파일)에서도 반응성 유지
- 취소/중단 용이

**구현 포인트:**
```
Core 데이터 구조:
struct Node {
    name: String,
    node_type: FileType,
    size: AtomicU64,  // None을 위해 u64::MAX 사용
    size_status: AtomicU8,  // Pending/Computing/Done
    children: Vec<NodeId>,
    parent: Option<NodeId>
}

스캔 흐름:
1. read_dir() -> 구조 생성 (동기)
2. 채널로 UI에 전송 -> 즉시 렌더링
3. 크기 계산 태스크를 워커에 분산
4. 완료된 노드를 채널로 UI에 스트리밍
```

### 방법 B: Progressive Loading
- 깊이 우선으로 스캔하되, 각 디렉토리마다 UI 업데이트
- 문제: 깊은 트리에서 중간 노드가 오래 비어보임
- OmniDiskSweeper는 이 방식이지만, 사용자 경험이 A보다 나쁨

---

## 2. 파일시스템 감시 (요구사항 3)

### 방법 A: notify crate + 이벤트 처리 (권장)
**선택 이유:**
- 크로스플랫폼 (inotify/FSEvents/ReadDirectoryChangesW)
- 효율적 (폴링 없음)

**구현 전략:**
```
1. 스캔 완료 후 감시 시작
2. 이벤트 수신 -> Debouncing (100ms)
3. 변경된 경로의 부분 트리만 재계산
4. 부모 노드들의 크기 재계산 (상향 전파)
```

**난관과 해결:**
- **대량 이벤트 폭주:** Debounce + 배치 처리
- **이동/이름변경 감지:** notify는 Rename 이벤트 제공, 하지만 플랫폼마다 다름. 안전하게 Delete+Create로 처리
- **순환 심볼릭 링크:** 스캔 시 visited 집합 유지, 감시는 심볼릭 링크 follow 안 함

### 방법 B: 폴링
- 주기적으로 mtime 체크
- 리소스 낭비, 배터리 소모
- 변경 즉시 감지 불가
- **비추천**

---

## 3. 코어-UI 분리 아키텍처

### 데이터 소유권
```
Core (별도 스레드):
- Arena 또는 SlotMap으로 트리 관리
- AtomicU64로 동시 읽기 안전
- 스캔/감시 로직 전담

UI (메인 스레드):
- Core의 읽기 전용 뷰
- 채널로 업데이트 수신 (crossbeam-channel)
- 렌더링만 담당
```

### 통신 프로토콜
```rust
enum CoreMessage {
    TreeUpdate { nodes: Vec<(NodeId, NodeSnapshot)> },
    SizeUpdate { node_id: NodeId, size: u64 },
    NodeRemoved { node_id: NodeId },
    NodeAdded { parent: NodeId, node: NodeSnapshot },
}
```

**Slint 연동:**
- Slint의 `Model` trait 구현
- `row_data_tracked()` 내부에서 채널 폴링
- 변경 시 `notify.row_changed()` 호출

---

## 4. 성능 최적화 고려사항

### 메모리
- 수백만 파일 시나리오: 파일명만 100MB+
- 전략:
    - String interning (같은 확장자 등)
    - 계층적 로딩 (보이는 것만 메모리)

### 병렬성
- Rayon으로 디렉토리별 병렬 스캔
- 단, 너무 많은 스레드는 FS I/O 경합 유발
- 경험적으로 CPU 코어 * 2 정도가 적절

### 플랫폼별 차이
- **Linux:** ext4는 readdir 빠름, btrfs는 느림
- **macOS:** APFS는 파일 수가 많을 때 느림
- **Windows:** NTFS는 첫 스캔 느리지만 캐싱 좋음
- 플랫폼별 최적화보다는 일반적 방법이 유지보수에 유리

---

## 5. 추천 크레이트
- `notify`: 파일시스템 감시
- `crossbeam-channel`: 멀티스레드 통신
- `rayon`: 병렬 스캔
- `dashmap`: 필요시 concurrent 맵
- `slotmap`: 안전한 노드 ID 관리

---

## 핵심 판단
- **2-Phase 스캔**으로 1번 해결
- **notify + 부분 재계산**으로 3번 해결
- **채널 기반 분리**로 UI 독립성 확보
- 정렬은 UI 레벨에서 처리 (Core는 정렬 안 함)
