# ProvGen 설명 계약과 P06-3 구현 대조

2026-10-07. 부모의 CEO 단일소유 연구 인수: `.orchestrator/ceo/research/provgen/ALGORITHM_REVIEW.md`, `review-receipt.json`, `README.original.txt`, `Settings.original.txt`, 최신 DIRECTIVES ProvGen절. 이 구현 세션은 제공 ZIP을 다시 조사하거나 실행/디컴파일하지 않았고 포함 BMP·경계·ID색·CSV·코드를 읽거나 인수하지 않았다.

연구가 고정한 ZIP은5,319,403bytes/SHA256 `62c8387f51562fb9983b8c230ac16b284a43176178abfba7b1d4948b69bc47d0`. 읽은 README는2039bytes/SHAbc4920d4658ca189619fd02aff177842755d3c3bd616d8337c11e176e458fd6a, Settings165bytes/SHA8be6eabe79d77da5a1f4b7dc9fc53412a948bf09347463590fd19b9e1fdfb8a4. source/LICENSE없음이라는 연구 한계를 유지한다. 무료 배포 문구는 코드/에셋 재사용 허가가 아니다. 날짜 문자열의 월/일 순서도 확정하지 않는다.

| README에서 확인한 제어 | 현재 독립 구현·검사 | 같은 것으로 주장하지 않는 부분 |
|---|---|---|
| 같은 크기 LandMap의 육지/바다/호수 구분 | 고정 NE polygon cell-center kind,4Kgraph/8Kshore transfer,allpixelkind·lakepriority·noanchor·4connected·seamadja 검사 | ProvGen 자체 섬/호수/해협 보존법·팔레트·내부 연결성 알고리즘은 미확인 |
| 육지 gray density 입력, 검정은 작은 크기·흰색은 큰 크기 | `coverage_seeds`의 sourcecomponent 구면 면적과 coast/city/river/validDEM density; 큰 source landmass minimumquota;12km physical seedexclusion | 현재 editablegray image/canonicaldensityformat/UIeditor 없음. 밝기→크기 함수·단위·시드 기법을 추측하거나 복사하지 않음 |
| 해상은 density image 대신 해안 거리 사용 | 독립 실제 coast physical spacing150km와250km밖의 ocean 후보, samekindgraph/actualwatermask·sea ID/hover/pick·sourceclass 보존 | 연안 세밀/대양 넓음은 사용자 지시와 우리 후보. ProvGen의 거리 함수 방향·단위·식으로 주장하지 않음 |
| 육해 min/max size 및 산악 범위 폭 설정 | defines의 nominalquota/spacing/meanurban-subdivision area와 actualridge/river/height costs, current2subsetDEM/누락 표시 | 예시 설정값·산악폭 단위·DEM/능선 추종·AverageState/상위해역 규격을 인수하지 않음. 우리 명목 평균/최소seedquota는 모든 결과 면적의 hardmin/max가 아님 |
| TerrainMap의 지형 분류·도시점 근처 전체province urban태그 | 실제 MODIS2002–03 footprint intersectland,majorfootprint separate rawIDs; point는 위치/seed 참고만; actualurbanpixels/sourcefraction 검증 | pointtag를 footprint와 혼동하지 않음. source밖 rural/suburban 권위 분류·1936city/population/VP/economy 미연결 |
| 공개 문서에는 조각 병합·결정론 방법 없음 | `repair_internal_fragments`의 제한된 connectedsame-surface 이동/seedcore·trueisland 보존·rawrecords, same final recipe 두bake4SHA 자체검사 | 이 코드가 ProvGen의 Voronoi/watershed/growth/merge/Lloyd 내부를 재현했다거나 반복 PASS를 설명서만으로 얻었다고 하지 않음 |

20,000~30,000 전체 목표는 사용자 요구이며 ProvGen 설정의 값을 이용한 목표가 아니다. 30,000을 절대 상한·육지 전용 목표로 확대하지 않는다. 현재 first clean d1de는26,213/7,139onepixel이었다. 후보3은 제한된 원클래스 보존 정리 후24,889/6,610onepixel이며 원천 지리·surface 섬이 남는다. 각 후보의 실제 city/sea/큰섬/면적·형상·선택·원실패를 별도 보고하고 총계 또는 ProvGen 비교만으로 quality/P05/mainCI를 판정하지 않는다.

모더가 회색조 이미지로 의도를 표시하는 입력은 후속 기술 검토 후보로 남긴다. 입력 버전·보간/밝기→단위·면적과 같은 topology/determinism/고정원천/원클래스 및 작은섬 보존 계약이 필요하다. 이번 작업에서 저장/프로토콜/정식mapformat/editor/D·OPEN·게임규칙을 새로 확정하지 않는다. 실제 최소 headless Chrome 검수·고정독립원천·원GLB·기존tests/checkers/guard·180초/3GiB를 유지한다.
