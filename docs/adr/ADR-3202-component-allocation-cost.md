# ADR-3202 연결 조각 정규화의 반복·좌표 할당 비용 절감

| 항목 | 값 |
|---|---|
| 상태 | 채택 — 자체 측정·새 독립 검증 대상 |
| 날짜 | 2026-10-07 |
| 관련 WP·REQ | WORLD-PREVIEW, REQ-MAP-10, 원 P05-3 F05 |

## 맥락

권위 리포트 WORLD-PREVIEW.M2-r2.attempt3.md는 exact446의 첫 fresh bake174.156s/native0와 다음180.078·180.188s/time-budget/native1로 유효 FAIL이다. 병행 회귀·화면 부하가 명시되어 있고 원 실패는 보존한다. 180s/3GiB/index64MiB/8K/4Kgraph를 변경하지 않는다. 이번 범위는 생성기의 계산·할당 비용이며 입력·defines·메타데이터·클라이언트·원 감독/검사는 변경하지 않는다.

먼저 원446 코드의 별도 대표 함수 입력을 직접 측정했다. 1024×2048의 동일 시드 분리8192조각은 connected_ids10.426649s, cProfile11.912456s 중11.877s가 반복문 자체였다. 큰 두 연결 영역의 첫 셀은 argwhere0.036961s/argmax0.001332s로 같은 좌표를 반환했다. shipped8K index를 시드로 취급한 연결 영역 대표는1.541286s이며 역사 rawseed 복원은 아니다. 모든 값은 단일 관측·함수 범위이며 cProfile overhead와 실제 전체 생성 시간을 구분한다. 증거: target/evidence/WORLD-PREVIEW-performance-P06-4/profile-original446.*.

## 결정

seed별 ndimage.find_objects 순서와 ndimage.label의4connected component 순서를 그대로 둔다. component마다 전체 bbox를 다시 비교·칠하는 반복을 count+1 길이 uint16 LUT 한 개와 parts!=0 마스크/np.copyto로 대체한다. 배경 label0은 쓰지 않아 먼저 처리한 다른 시드의 셀을 덮지 않는다. 순서대로 next_id..next_id+count-1을 할당하며 합계65535를 넘으면 LUT·출력 쓰기 전에 u16 ID overflow로 거부한다. 표시 ID1..65535/ordinal0..65534, 빈 seed slot·입력 rowstride·signed/unsigned dtype을 유지한다. 전역 raw+1/라벨 배열 dtype과 source/RNG/고정 선택 순서를 변경하지 않는다.

repair에서 firstcell 하나만 쓰는 모든좌표 argwhere를 bool mask의 C-order argmax와 divmod로 대체한다. component bbox는 find_objects가 반환한 비어 있지 않은 영역이며, original argwhere의 첫 행 우선 좌표와 같다. 이동 조각의 실제 모든 cells 추출/nonzero, seed별 순서, stable target/edge/크기/ordinal 동률 순서와 records/debug 카운터는 그대로 둔다.

LUT는 최대65536×2byte, mapped bbox uint16와 bool mask를 잠시 만든다. 이를 전체 memory 절감 보장으로 표현하지 않고 원 supervisor로 실제 PeakCommit/WorkingSet를 확인한다. 전체 kind/urban/eligible class·기하·ID·면적·대표DEM·adjacency와 메타데이터까지 원446 네 파일의 exact bytes를 우선한다. 변경된 코드 이름 때문에 metadata/tools/options/source/epoch 값을 바꾸지 않는다.

## 검토한 대안

| 대안 | 장점 | 선택하지 않은 이유 |
|---|---|---|
| 원 반복 유지·시간 한도 확대 | 코드 유지 | 원 예산 검사를 대체함 |
| 모든 시드 연결을 한 번에 relabel | 반복 축소 가능 | 시드/component 순서·배경 semantics가 달라질 수 있음 |
| bbox마다 uint16 LUT와 조건부 copy | 순서·dtype·배경 유지, 조각 수와 곱해지는 비교 제거 | 선택; 메모리는 실제 측정 필요 |
| 모든 component 좌표를 미리 전역 저장 | firstcell 재사용 | 불필요한 전체 좌표와 추가 전역 메모리 |
| 비어 있지 않은 bool mask 첫 참 셀 | 원 첫 좌표 유지 | 선택; 실제 전체 bake 효과는 별도 측정 |

## 결과와 영향

오프라인 제작 계산만 바꾸며 게임 DR/권위/저장/프로토콜·도시/해상/작은 섬/수로 규칙을 변경하지 않는다. 새 패키지·설정·CPU/GPU/사용자 프로세스 변경 없음. 현재 설치된 Python/NumPy/SciPy와 원446 metadata version을 그대로 사용한다. 새로운 버전·라이선스 주장을 추가하지 않는다. 독립 floodfill/빈 slot/멀리 떨어진 조각/다양한 dtype·stride/65535·65536/비교 작업량·첫셀 할당 RED→GREEN과 원 기존 검사, 같은 소스 fresh 두 번의 실제 제한·4SHA, 화면·전체 회귀를 남긴다. 자체 통과를 독립 PASS/main CI·마일스톤 완료로 기록하지 않는다.

## 실제 후속 관측

동일 프로파일 helper의 old/current8192조각 출력 SHA는 같고 current0.037513s다. 모든 경우의 보편적 가속은 아니다: shipped dense index를 시드로 사용한 대표는 old0.883741/current0.997251s로 current가 조금 느렸으며 원 rawseed 복원으로 해석하지 않는다. 큰 전체-coordinate 할당 제거는 별도 argwhere0.034468/argmax0.000943s와 동일 첫 좌표를 확인했다. 단일 함수 관측·프로파일 overhead와 전체 생성은 구별한다.

새 실제 offline fresh1/2는72.438·71.594s/native0, PeakCommit2,639,233,024·2,640,277,504B/WorkingSet1,638,522,880·1,639,227,392B였다. 원180s/3GiB/64MiB 감독은 동일하며 시간 여유107.562·108.406s. 원446/각fresh 네 파일이 metadata까지 exact bytes이다. pair 중 이 세션은 cProfile/build/test/browser를 병행하지 않고 작은 문서·hash 작업만 했으며 own Vite는 idle이었다. 다른 앱·사용자 부하는 측정하거나 제어하지 않았다. 이전 독립 검증은 일부 회귀/E2E/화면 병행을 명시했으므로 이 시간 차이를 엄밀한 동일 부하 benchmark 또는 모든 환경 보장으로 주장하지 않는다. 원174.156/180.078FAIL/180.188FAIL은 보존한다.
