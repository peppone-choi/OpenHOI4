# 전세계 지도·프로빈스·메인 HUD 우선 미리보기

## 현재 체크포인트와 다음 표시 품질 개선

### 최신 사용자 제작 목표 — 아일랜드·도시·해상 밀도

사용자는 아일랜드가 한 프로빈스인 화면을 지적했고, 전세계 목표를 "적어도 20000~30000개", 필요 도시 시가지를 별도 프로빈스로 분리, 바다도 고유 프로빈스 단위로 구획하라고 명시했다. 전체 land+sea+lake의20k~30k(최소20k)로 인수하며 육지만의수로 확대하거나30k를새절대상한으로확정하지않는다. 기존기술u16 max65535는유지한다. 도시/해상전투·경제·항만·상위전략해역게임규칙은이번외관생성요청이아니다.

부모의ab64 data직접4connected계수에서 (-8,53)의아일랜드본섬은5789pixels/ID6809 하나·largestshare1.0/cropedge불접촉이었다. 북아일랜드를포함하는지리적연결육지이며국가소유경계가아니다. sourceSHA와[원ab64 봉인](../worklog/evidence/WORLD-PREVIEW-M2-r2-ab64/README.md)을보존한다. 전체20366=land10233/sea6080/lake4053 중1pixel2639/2369/1492=6500을따로집계한다. 숫자만으로적정밀도나품질을선언하지않으며모든singlepixel삭제·단절섬merge·의도미세조각으로총계채우기는하지않는다.

같은지도담당P06-3에서 Ireland/UK/Japan/Europe와대륙별큰육지component의ID수·면적/장축분포/seedcoverage를먼저계수하고공통지리분할로개선한다. 도시point추가만으로도시분리했다고쓰지않고독립built-up footprint·강/해안/지형을검사한다. 도시행정경계전체를시가지로자동취급하거나현대좌표를1936인구/VP/경제로인수하지않는다. 모든도시1개/전도시내부분할은사용자확정규칙이아니다. sea는고유ID/경계/영역/인접/hover/select, lake는별도kind로유지하고연안/해협/섬주변세분·대양의상대넓은구획/수로연결을검사한다. 상위전략해역group은새규격확정으로확대하지않는다.

공개본편/모드의aggregate province수 조사는 최신사용자의한정예외다. 출처·버전·land/sea포함범위/역사·community/port값을구분해기록하고원CSV/지도/경계/ID색/게임수치표를다운로드·재사용하지않는다. 숫자를그대로생성규칙에복사하지않고독립지리·도시footprint와지역별실질세분/클릭가독성을기준으로한다. 조사원문은CEO province-count-public-research-20261007.json과DIRECTIVES에보존한다. 새같은축척 Ireland/Dublin/EnglishChannel/IrishSea/Japan근해·대양 실제beforeafter를먼저보여주며원ab64를교체하지않는다. 최신새source는독립검수가필요하고새게임기능은계속후속이다.

실제 세계/HUD e209·실3D a28와 지리 자료/Fluent 정합 `2d920ea86c6d8045adb7d3115007230e52c3ccf2`를 각각 고정 증거로 보존했다. 최신 [원 화면·입력·95파일 봉인](evidence/world-preview-checkpoint-2d920/README.md)은 최소인자 전용 headless Chrome의 실제 WebGPU/WebGL2 페이지다. 13413개 독립 프로빈스, NaturalEarth 하천 및 히말라야 일부 NOAA DEM, 실제3D 샘플6개/912triangle과 미연결 HUD를 표시한다. 원 `ERR_ABORTED`4개는 cleanup/navigation 취소 여부를 독립 대조하고 HTTP/page 실패와 합쳐0으로 쓰지 않는다.

원a28 독립 [유효 FAIL](../verify/WORLD-PREVIEW.M2-r2.attempt1.md)은 F01 Fluent68누락 및 F02 단일전체 E2E 녹색 미충족이다. 첫검증9999/full/rawaf9986 불변·MF533/ZIP534 전수0, 원ZIP db5f74...을 보존했다. 새2d920 현지화 수정·캡처를 그 원판정에 섞지 않으며 새 source는 새 독립검수 대상이다. a28 baked입력과 최신 입력의 전후 비교는 같은최종renderer/카메라의 지오메트리 비교이며 원a28source 자체를 다시 검증한 것은 아니다.

부모와 CEO의 실제 한반도 확대 관찰은 큰 해안 pixel계단·반도폭을 가로지르는 소수 큰프로빈스다. 사용자 요청의 지역별 형태·밀도 품질은 아직 부족하다. 같은 지도 담당이 원 체크포인트를 먼저 봉인한 뒤 지역밀도·해안/하천/능선 기반 경계·표시해상도를 다음 개선으로 진행한다. 8k 이내 후보의 실제 생성시간·메모리·GPU texturelimit·selection/ID/연결성/인접/seam/반복SHA와 같은축척 전후 화면을 대조한다. 숫자는 지도제작용 defines이며 게임수치가 아니다. 없는 도시/고도 원천을 발명하거나 원 경계를 복사·trace·추출하거나 임의noise로 품질을 대신하지 않는다. 전세계 DEM·도시밀도·정밀능선 및1936콘텐츠는 미완료로 유지한다. 새 게임 기능 연결은 계속 후속이다.

2026-10-07. 사용자 직접 지시로 제작 순서를 변경했다. 첫 결과는 실제 전세계 윤곽과 독립 생성 프로빈스 경계, 메인 HUD 외관이다. HUD의 새 게임 기능은 이후 권위 데이터와 연결한다. 기존 WP-13 P-06 오류 수정은 계속하며 M2 전체 기능과 M5 전체 역사 콘텐츠 완료를 기다리지 않는다.

라이선스를 확인한 버전 고정 지리 원천을 로컬 파일로 취득하고, 오프라인 생성기가 프로빈스 인덱스·메타데이터·출처 및 입력/산출 hash를 만든다. 기존 Three.js 지도 렌더러를 재사용한다. 런타임은 동봉 자료만 읽는다. 전세계와 확대 지역의 실제 브라우저 캡처, 카메라 보기 조작, HUD 미연결 범위, 정확한 checkpoint 커밋·로컬 URL·실행법을 첫 결과부터 보고한다. DEM·하천·도시의 실제 적용과 누락은 구분한다.

최신 사용자 지시에 따라 computer-use/CUA·사용자 화면·브라우저 포커스·키마우스 조작은 금지다. Breaking Point UI 세션의 실제 방식을 읽기 전용으로 확인해, 사용자 작업을 방해하지 않는 별도 headless/background 렌더링과 캡처로 검수한다. 이미 읽은 skill보다 사용자 지시를 우선한다. 기존 사용자 탭·프로세스·가속/보안 설정을 바꾸지 않는다.

CEO가 실제 Breaking Point의 별도 native desktop/PID·PrintWindow·전역 activation/input 없는 캡처를 확인했다. 그 프로젝트 helper/게임/자료/desktop/PID는 복사·재사용·실행하지 않는다. OpenHOI는 자기 localserver/profile/PID의 headless/background Chromium+Playwright/CDP page 캡처에 같은 분리 원칙을 적용한다. 검은 canvas는 실패이며 foreground fallback하지 않는다. 실제 backend/headless/PID·명령과 가능한 읽기 전용 foreground/cursor 전후 관측을 보존한다. 사용자 자체 입력으로 값이 변하면 임의 restore하지 않는다.

임시 유닛 마커는 built-in imagegen으로 생성한 투명 PNG를 사용한다. 샘플 배치는 실제 군대나 게임 상태가 아니다. 원 생성 prompt·도구·제공 모델 정보·조건·일시·SHA와 교체 가능한 leaf 파일을 보존한다. 세계 지도나 HUD의 실제 렌더링을 생성 이미지로 대신하지 않는다. 공개 배포는 별도다.

이후 사용자 직접 3D 요청으로 지도 위 마커는 자체 procedural low-poly 전차·항공기·함정 GLB 메시로 전환했다. 동일 에셋 앱이 assets/preview/units3d 및 client/public/preview/units3d의 GLB·생성 recipe·단위/pivot/방향·normal/indices/material·bbox/재생성 SHA·entry 초안을 소유한다. 이미 생성된 imagegen PNG와 실패/수정 원본은 그 branch와 증거에 보존하지만 3D 모델로 표시하지 않는다. 세계 지도 앱은 제출된 GLB leaf 원문을 인수하고 실제 scene/depth/light와 표시용 transform만 연결한다. 미검토 PNG branch 전체를 병합하거나 공개 배포 승인으로 해석하지 않는다. 실제 군대/배치/명령은 여전히 후속이다.

| 작업 | 앱·전용 worktree·브랜치 | 단일 소유 |
|---|---|---|
| 세계 지도·HUD | `01a11536-8218-7c93-9082-70931a350875` / `.orchestrator/wt/WORLD-PREVIEW-M2-r2` / `codex/world-province-preview-m2r2` | 신규 tools/maps·data/preview/world·client preview/동봉 자료·작은 App route·preview 현지화, ASSETS master 신규 append |
| 생성형 마커 | `01a11537-ccb8-7f80-938d-6b86803b1bf2` / `.orchestrator/wt/PREVIEW-MARKERS-M2-r2` / `codex/preview-marker-assets-m2r2` | client/public/preview/markers 3 PNG 및 assets/preview/markers 원문/manifest entry 초안. ASSETS master 수정 금지 |
| 같은 에셋 앱의 최신 3D 소유 | 위 동일 앱·worktree·브랜치, 추가 앱 없음 | assets/preview/units3d 및 client/public/preview/units3d의 실제 GLB·recipe/metadata/entry 초안. 신규 raster 호출 중지, 원 PNG 보존 |
| 진행 중 WP-13 P-06 | `01a114e4-115e-7823-ba09-966b6efb7594` / `.orchestrator/wt/WP-13-M2-r2` | 기존 인계 network/Trigger checker·회귀 및 자기 문서. 지도/HUD와 겹치지 않음 |

preview 두 구현의 실제 base는 `ea63081e0b79369227c99384922db80c6ef09fb1`이다. 마커 PNG/원문은 부모가 세계 지도 담당에게 정확 leaf 경로·bytes/SHA로 인계한다. 세계 지도 담당이 ASSETS 등록과 화면 연결을 한다. 실제 사용자 원문·create payload/prompt/result·첫 active 관측은 `.orchestrator/evidence/M2-r2-resume/`에 보존했다. 고유 통합WP14·원W1/W2 전체 통합0·M2 게이트 미판정은 유지한다. preview 작업을 원 M2 병렬 묶음 통합으로 세지 않는다.

원작 지도·아이콘·국기·상징·파일·스크립트·수치표 복사와 tracing은 금지다. 원Testland/M0/M1·골든·저장 fixture·기존 codec/워크플로·D/OPEN 상태를 보존한다. 새 역사 국가·소유권·경제·전쟁 규칙과 HUD 권위 명령은 범위 밖이다. 별도 정확 checkpoint의 새 독립 시각 검수와 source 불변성을 얻고 일반 M5 게이트 완료와 구별한다.

최신 사용자 프로빈스 형상 요청은 공개 플레이 화면에서 지역별 크기·장축·굴곡·산맥/강/해안/도시 주변 분할 원칙을 출처 URL과 함께 관찰하는 범위로 인수했다. 원 경계 좌표/파일 복사·tracing 없이 실제 생성 경계는 독립 지리·DEM/하천 자료에 근거해 개선한다. e209의 균일 nearest-seed 화면은 [첫 실제 checkpoint](evidence/world-preview-checkpoint-e209/README.md)로 보존하고 최종 형상으로 확정하지 않는다. 임의 굴곡 noise로 지리 근거를 대신하지 않으며 같은 축척 지역의 전후 실제 렌더링·연결성/조각/인접/재생성 hash를 대조한다. 이미지 편집으로 모더 부담을 줄이자는 별도 질문은 의견 검토이며 제품/schema 계약을 변경하지 않는다.
