# WP-33 W-1 세계 규모 MapView 계측 (비제품)

- taskId: t_5b8653e8, project: openhoi, stage: implement
- base: main 092938b5aec9b50ad056205c8066987d420f1243
- branch: impl/t_5b8653e8. PR28과 독립이며 제품 코드·데이터·기존 시험은 변경하지 않는다.
- nonProduct=true, productComplete=false. actualProductPack/gameplay/M1/AC10/productQA NOT_RUN.

## 입력과 신뢰 경계

커밋된 preview 8192×4096, 24,889개(land 13,882/sea 6,954/lake 4,053)를 쓴다.
metadata.hashes의 index.bin·provinces.json SHA-256을 적재 전에 검증한다.
index.bin 고정 해시는 `3aafdad84138670ea627535952b987d16e79519fe33fe617a6c7321917358706`이다.
metadata.json에는 자기 해시가 없다. metadataObserved는 관측 해시이며 신뢰 검증 해시가 아니다.
기존 validatePreview와 고정 치수·개수 검사도 수행한다. 독립 32-bit limb FNV와 제품 BigInt FNV를 비교한다.
오류 주입 회귀시험은 payload 해시 불일치의 파일 이름 및 알려진 FNV 벡터를 검사한다. 원격 broken commit은 만들지 않는다.

실제 oh_server의 WorldResult에서 tick/neutral_color/mode_keys/terrain_key를 가져온다.
국가·주는 비어 있고 소유·통제·주 참조와 색은 null이다. geography_color는 terrain 표시 placeholder이며 Q1 정책이 아니다.
실제 metadata의 schema_version/pack_hash/style을 유지하고 지도 식별자·치수·프로빈스·바이트 길이·독립 FNV만 바꾼다.
실제 App이 수신하는 첫 WorldResult를 교체하고 후속 WorldResult는 억제하여 정지된 표시 스냅샷을 잰다.
후속 억제 개수는 기록하며 다른 서버 메시지는 그대로 전달한다. 게임 진행·서버 세계 팩을 시험하는 것이 아니다.

## 실행과 증거

새 workflow `world-scale-browser.yml`은 workflow_dispatch와 관련 경로 PR 이벤트를 사용한다. required check는 아니다.
Chromium만 사용하며 시험당 15분/job 45분은 실행 상한이지 성능 합격 기준이 아니다.
Node 24.4.0/Rust 1.99.0, 실제 제품 build와 oh_server, 전용 19429 포트, reuseExistingServer=false이다.
VPS에서 빌드·계측·브라우저를 실행하지 않는다.

- N1: Node 내장 WebSocket으로 실제 Hello/Join/Query를 수행해 입력을 취득한다. 브라우저 fixture 없이 validateWorldDisplay/decodeIndex/fnv/palettes(owner)/updateColors(terrain)/provinceAnchors 각 3회.
- validateWorldDisplay/palettes/updateColors는 n=3,111/6,222/12,444/24,889 각 3회 및 log-log 기울기. 시간 임계값은 없다.
- B1: 첫 주입부터 2프레임·loading 소멸까지 wall ms, 실제 backend/texture limit/frame ms, CDP Performance metrics 전후.
- B2/B3: terrain 전환·가장 큰 land의 raster anchor 선택, wall/update ms.
- B4: 가장 큰 land/sea의 raster anchor를 확대하고 입력 raster 내부 5×5 화면 이웃 점에서 geography_color와 스크린샷 RGB 비교(채널별 오차 ≤1). 제품 렌더러에서 기대색을 가져오지 않는다.
- 실제 HTTP/WebSocket 요청의 host/protocol을 검사한다. 지도 metadata/index 요청을 확인한다.

함수 호출·단계 완료 때마다 JSON을 저장한다. 실패/시간초과는 FAIL이며 앞 단계 수치는 유지한다.
MapView는 texture-limit 원인을 backend-unavailable로 합치므로 UI가 노출하지 않는 세부 원인을 발명하지 않는다.
오류 분류, 마지막 단계, 경과 시간, 브라우저 버전, exact SHA, runner, screenshot, trace/report/server log를 항상 업로드한다.
Linux supervisor는 자신이 시작한 프로세스 자손의 PID+시작 시각을 기록하고 성공·실패·시간초과 시 소유 프로세스만 종료·부재 확인한다.
후속 always 검사에서 잔존이 발견되면 정리 후에도 CLEANUP_FAILED이다. checkout 변경도 실패로 처리한다.
시작 전 PID+시작 시각 baseline과 비교해 새로 남은 서버·브라우저·Playwright 관련 프로세스도 검사한다. 소유 확인이 안 된 프로세스는 종료하지 않고 실패로 보고한다.
하드 runner 소실은 finally 실행을 보장할 수 없으며 GitHub가 runner를 폐기하는 경계를 따른다.

## 실제 첫 실행 및 수정

`e6c2b03229d75fc30576148e356a6a894a54fd5e`, hosted run 39130455066에서 build/신뢰 경계/Node 계측/브라우저 적재·전환·선택은 성공했다.
픽셀 단계는 축소한 세계 화면에서 가장 큰 land(ID18014)의 5×5 내부 화면 점을 찾지 못해 실제 FAIL했다. 부분 JSON·스크린샷과 정리 PASS가 업로드되었다.
시험은 해당 입력 anchor에서 확대 후 내부 점을 찾도록 수정했다. RGB 허용치·입력·제품 코드는 그대로다. 최종 재실행 결과는 RESULT.json을 따른다.
두 번째 run 39130839035는 초기 DOM 조회의 1초 locator timeout이 상위 polling을 중단한 harness 오류였다. 이후 진단에는 정상 WebGL2 프레임이 기록되었다. 초기 미표시는 polling의 미준비 상태로 처리하도록 수정하고, 15분 시험 상한·실제 지도 alert 실패는 유지했다. 이 실패도 증거에 보존한다.
세 번째 run 39131250905는 3개 시험·정리·입력 RGB 비교 모두 성공했다. 최종 자체 점검에서 Node 입력 취득에도 브라우저가 쓰인 계약 불일치를 찾아 Node 직접 WebSocket으로 교체했다. afterEach는 실제 최종 시험 상태로 부분 JSON을 마감한다. 이 후속 변경의 exact-head 재실행을 별도로 확인한다.

## 인수 및 한계

신뢰 경계·fixture null 참조·계측 횟수·실제 경로·동일 출처·픽셀·부분 증거·정리·허용 diff는 독립 리뷰 대상이다.
실제 hosted 실행의 exact SHA/run URL/결과는 외부 `results/t_5b8653e8-RESULT.json`에 기록한다.
GitHub-hosted 소프트웨어 렌더링 측정이며 실제 GPU 제품 성능이 아니다. 최적화·성능 임계값은 후속 별도 결정이다.
주별 filter는 NOT_MEASURED(Q1). 사단 마커·이동·주 경계는 NOT_RUN(Q1).
독립 리뷰·제품 인수·main 병합은 이 구현자가 수행하지 않는다. 다음 담당 openhoi-review.
