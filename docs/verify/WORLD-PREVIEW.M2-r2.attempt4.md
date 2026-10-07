# 검증 리포트 — WORLD-PREVIEW-M2-r2-P05-4

| 항목 | 값 |
|---|---|
| 판정 | **PASS — 세계지도·HUD·실제3D 미리보기 한정 독립 검증** |
| 검증자 | 구현 비참여 새 Codex 앱 검증 세션, 추적 파일 무변경 |
| 브랜치·커밋 | detached `5794505abb2b0cfeaace8005183cb6606fc68301` |
| 검증 일시 | 2026-10-07 KST, 최초 기준선 확인 22:45부터; 실제 각 실행 시각은 receipt에 기록 |
| worktree | `E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify4` |
| 범위 | P06-4 비용 개선·원 산출물 보존, 지도 coverage·major 시가지·해상·표시·회귀 |

새 정확 source에서 생성·원천·화면·회귀 계약을 직접 확인했다. **이 PASS는 WP-32/45 전체, M2/M5, 1936 역사 콘텐츠, 게임 기능, 사용자 외관 승인 또는 기본 브랜치 CI 완료가 아니다.** 역사 P05-1 F01/F02, P05-2 F03/F04, P05-3 F05의 유효 FAIL을 변경하지 않는다.

## 직접 실행한 명령

[자기 증거 루트](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify4/target/evidence/WORLD-PREVIEW-M2-r2-P05-4)에 실제 argv/cwd/PID/시작·종료 UTC/native exit/stdout/stderr를 보존했다. 셸 명령은 지정 worktree와 `GIT_OPTIONAL_LOCKS=0`에서 실행했다. 최초 bake의 **바깥 기록 wrapper만 cwd 계산 오류로 상위 `.../.orchestrator/wt`에서 감독을 시작했다.** 원 receipt를 유지했다. 감독은 절대경로의 입력·출력을 받아 실제 generator를 지정 verify4 cwd에서 실행했다. 그 뒤 wrapper를 바로잡고 추가 fresh3까지 실행해 fresh2/3의 올바른 실행 경로도 확인했다. 이 예외를 숨기고 모든 subprocess cwd가 같았다고 쓰지 않는다.

| 실제 명령·검사 | native exit | 직접 결과 |
|---|---:|---|
| `npm --prefix client ci`, `run typecheck`, `run build`, `run licenses` | 각 0 | 통과; 103개 dependency license 검사 |
| `npm --prefix client test` | 0 | **단일 전체 140 PASS** |
| `python -m unittest discover -s tools/maps -p test_*.py -v` | 0 | **33개 실행, 32 PASS·1 기존 skip**, 새 비용 테스트 6개 포함 |
| `python tools/check_docs.py`, `tools/check_assets.py --release`, 기존 `tools/check_localisation.py` | 각 0 | docs/assets/Fluent 통과, 기존 경고는 원 로그 유지 |
| 절대 Cargo의 `fmt --check`, `clippy --workspace --locked -- -D warnings`, `build --workspace --locked`, `test --workspace --locked` | 각 0 | **179 PASS**, 저장·재개·새 native process·read-only generated TS equality 포함 |
| `python tools/check_architecture.py`, `cargo deny check licenses bans advisories sources`, Testland `validate --deny-warnings` | 각 0 | 통과 |
| M0 `run --scenario testland --days 365 --seed 1 --hash-out` 두 번 | 각 0 | 같은 해시 |
| M1 `run --pack data/packs/testland --scenario m1 --days 365 --seed 1 --hash-out` 두 번 | 각 0 | 같은 해시 |
| `tools/maps/bounded_bake.py --source <absolute fixed source> --out <new ignored fresh1/2/3> --receipt <new file>` | 각 0 | 아래 시간·메모리·4SHA 확인 |
| 독립 SHP 재래스터화·면적·모든 ID·urban·DEM·인접, 자체 600개 floodfill/stride/첫셀 oracle | 각 0 | 전체·경계 검사 통과 |
| F03/F04·noanchor·fragment counter fixture, source metadata/ZIP CRC·Pillow TIFF 대조 | 0 | 통과 |
| ignored 입력 mutation 5종·Fluent 키 제거 | 각각 **1** | 예상 거부; 검사 helper 0, 원 입력 bytes 보존 |
| Node GLB recipe ignored 재생성·3축 geometry/CCW/normal/material·fragment 전수 대조 | 각 0 | 통과 |
| 자기 최소 headless Chrome 화면·client negative·3D occlusion·21개 카메라 비교 | 각 0 | browser native exit0, 실제 frame·선택 확인 |
| 원 M1 전체 Chrome E2E | 0 | **단일 41 PASS / 0 skip·flaky·unexpected** |
| 원 M1 전체 Firefox E2E | 0 | **단일 41 PASS / 0 skip·flaky·unexpected** |
| 원 M1 전체 WebKit E2E | 0 | **단일 41 PASS / 0 skip·flaky·unexpected** |
| producer ZIP/MF·critical/artifact·actual final 및 역사 ZIP 1~3 전수 대조 | 0 | 원 bytes/SHA/CRC 일치 |
| 부모→자기 최초·중간·마지막 full/raw 비교 | 각 0 | 전수 동일; 봉인 뒤 비교는 아래 원 receipt 참조 |

원 tracked bake/TS writer는 호출하지 않았다. E2E의 config/testMatch/workers1/timeout/retry/assertions는 그대로이며 ignored config에서 자기 경로·포트·출력과 Chrome 최소 인자만 지정했다. Chrome E2E의 `hide-scrollbars`는 원 폭 기대값을 위한 cosmetic 옵션이다. security/sandbox/GPU override 또는 foreground fallback은 없다.

Python skip은 과거 ignored 취득 경로 부재다. 고정 두 TIFF는 별도 Pillow decoder로 **모든 pixel equality**, finite/range/extent와 모든 대표 DEM-or-null을 확인했으므로 skip을 성공으로 세지 않았다.

### 실제 생성 성능·원 산출물 보존

| fresh | 실제 generator PID | 감독 elapsed | native | PeakCommit B | PeakWorkingSet B |
|---|---:|---:|---:|---:|---:|
| 1 | 16636 | **59.047s** | 0 | 2,640,867,328 | 1,639,669,760 |
| 2 | 36284 | **59.047s** | 0 | 2,639,204,352 | 1,638,850,560 |
| 3 추가 | 34216 | **57.828s** | 0 | 2,640,162,816 | 1,639,620,608 |

각 이전 종료를 확인한 뒤 순차 실행했다. 자기 build/npm/Rust/browser/픽셀 검사/cProfile 작업은 세 측정 동안 병행하지 않았고 자기 Vite도 아직 없었다. 부모 Git commit/push 가능성과 사용자·외부 앱 부하는 미측정·미제어다. 시스템 전체 무부하나 모든 환경 성능을 보장하지 않는다. 원 감독·limits **180초 / 3GiB / index64MiB / 8K coast / 4K graph**는 그대로다. OS own-PID peak counters와 전기간 samples, wrapper 시작·종료 시각, 실제 argv와 native exit를 보존했다.

first wrapper cwd 이탈은 실제 generator의 cwd·입력·출력을 바꾸지 않았다. 절대경로·감독 source를 직접 대조했고 최초 이후와 마지막 full/raw 불변성이 같다. fresh1을 삭제하거나 원 최초 pair를 새 pair로 덮지 않았다. 추가 fresh3는 이 경로 오류의 보완 증거다.

세 fresh의 index/provinces/adjacency/metadata는 서로, 새 shipped와 역사 `446540f...:client/public/preview/world/` Git blob까지 **exact bytes**가 같다.

| 파일 | bytes | SHA256 |
|---|---:|---|
| index.bin | 67,108,864 | `3aafdad84138670ea627535952b987d16e79519fe33fe617a6c7321917358706` |
| provinces.json | 7,116,532 | `f0f8efca5086d51580b7ffeeee862f39f3edef19ae3fac0a75091b6cdc1c3c91` |
| adjacency.json | 617,874 | `2a44a5e0132840f86e37d2df9c08087fdac2d5fc0ab2d38921106af44d7247fa` |
| metadata.json | 622,581 | `0498d827389750a32e37dca196e54bb5f15c8104a2a342611b1f8c135c648dbf` |

446 대비 변경은 generator 비용 코드20줄, 새6테스트, ADR-3202와 append 작업 로그 네 파일뿐이다. 입력7·defines·client·GLB·proto/Rust·dependencies·CI·원 테스트·골든은 그대로다. component ordinal LUT는 background0을 덮지 않고 seed/component 순서와 uint16·65535 허용/65536 거부를 유지한다. first-true argmax는 guaranteed nonempty component bbox의 원 row-major 첫 좌표와 같다. 별도 4neighbor oracle 600개에서 transpose/stride·빈 seed slot·배경 보존을 대조했다. 함수 미시 벤치마크10.426649→0.037513s를 전체 생성 측정으로 인용하지 않는다.

## REQ별 확인

| REQ·계약 | 직접 검사·테스트 | 결과 |
|---|---|---|
| REQ-MAP-01/02/07 일부 | fixed known coordinates, RG8/u16LE·dense ID·4connected·bbox·rep·seam | PASS |
| REQ-MAP-04/05 일부 | 원 각 browser41, raw-index CPU pick·hover/select/clear·zoom/pan·resize/DPR | PASS |
| REQ-MAP-10 | 고정7 source·CRC/metadata·거부·독립 offline fresh 반복·예산 | 한정 PASS |
| REQ-MAP-11 일부 | 실제 river/DEM cost·emptyCSR/edgeless/fallback·대표 DEM-or-null | 한정 PASS |
| coverage·major urban·sea/lake | 전체 sourcekind/eligible mask·구면 면적·실제 boundary/ID | 한정 PASS |
| REQ-LOC-01/03·F01 | 기존 Fluent checker·실제 KO/EN·키 제거 거부 | PASS |
| 원 renderer·F02 | 새 정확 source의 Chrome/Firefox/WebKit 단일 전체41 각각 | PASS |
| F03/F04 | diagonal-only 유효2×2, noanchor1×4 floodfill, 65535/65536·실제 client validator | PASS |
| 실제3D·HUD·REQ-NET-04 | GLB/scene/depth/cleanup·미연결 슬롯·in-memory TS equality | 한정 PASS |

### 전체 지도·지역 coverage·시가지·해상

**8192×4096, 24,889 ID = land13,882 / sea6,954 / lake4,053.** 모든33,554,432pixel kind는 별도 SHP decoder/scanline raster와 같고, 모든 ID는4connected·dense·bbox·대표좌표 유효이며 sorted unique adjacency47,554쌍에 seam이 포함된다. 별도 source polygon∩land urban과 >=100km² eligible class도 전체 일치했다. 도시7342점 중 sourceurban 밖4119는 누락으로 남긴다.

구면 면적은 defines R=6371.0088km와 `R²Δλ(sinφN−sinφS)`로 재계산했다. 최대 반올림 오차 약4.99967e-7km². Plate Carrée pixel수를 km²로 부르지 않고 이 값을 국가 면적·측량값으로 쓰지 않는다.

| kind | 전체1pixel province | sourcekind 자체1pixel 조각 | <=8pixel province | 면적 p50 / p90 km² |
|---|---:|---:|---:|---|
| land | 2769 | 2366 | 5586 | 315.89 / 34950.81 |
| sea | 2349 | 2267 | 4125 | 42.28 / 208121.85 |
| lake | 1492 | 1492 | 3065 | 33.38 / 370.68 |
| 합계 | **6610** | **6125** | **12776** | kind별 분리 보고 |

추가1pixel485와 전체 tiny를 자연 섬 개수로 표현하지 않았다. 총수만20k~30k라는 이유로 품질 PASS하지 않았고 작은 원천 지리를 삭제하거나 단절 geography를 합쳐 수를 맞추지 않았다. 30k를 절대 상한 또는 육지만의 목표로 만들지 않았다.

Ireland(-8,53)의 동일5,789pixel/약82,360.65km² 연결섬은 ab64의 **1ID6809 → 17ID**, 13개>=9pixel과 tiny[1,2,2,8]이다. largest pixelshare17.17%, 면적share17.11%. GB68 / Hokkaido16 / Shikoku10이며 Honshu·Kyushu는 원천에서 연결된173ID component다. 실제 Kyushu 수로와 Caspian 누락 한계를 임의 경계로 보완하지 않았다. 75개 큰 landcomponent의 공통 면적 minimum quota·landseed6000을 검사했고 Ireland 특례가 없다. Eurasia/Africa·North/SouthAmerica source component와 Europe/Africa/EastAsia/NorthAmerica/SouthAmerica 지리 window를 구분했다. window IDs2195/1575/1809/3189/1254는 국가·대륙 전체 계수가 아니다. actual IDs·면적 분포·큰셀 편중·도시점 coverage는 coverage.json과 geography-extra-audit.json에 보존했다.

major 도시분리는 >=100km² **1846component**이고 전체 footprint11967component/57247pixel이다. 작은 footprint는 미분리다. point seed 증가를 footprint 분리의 증거로 삼지 않았다.

| 도시 | actual source point 선택 ID | parcel pixel / 근사km² | source footprint pixel / 근사km² | urban fraction |
|---|---:|---|---|---:|
| Dublin | 23023 | 24 / 342.17 | 24 / 342.17 | 1.0 |
| London | 23080 | 161 / 2392.75 | 187 / 2778.85 | 1.0 |
| Seoul | 23751 | 136 / 2566.56 | 341 / 6459.56 | 1.0 |
| Tokyo | 23828 | 80 / 1550.20 | 968 / 18700.46 | 1.0 |

실제 source point·reference 거리0.415/0.235/0.321/0.776km, parcel bbox·4neighbor ID·주변 nonurban을 기록했다. Tokyo 선택 parcel은 내부 urban parcel이어서 바로 접한 nonurban ID가0이며 화면의 주변 footprint 밖 영역과 구분했다. 모든 도시 parcel에 nonurban 직접 인접이라는 새 규칙을 만들지 않는다. 동일 zoom60 도시 PNG/hover/pick은 raw index와 같고 source밖 영역을 권위 농촌/교외로 부르지 않는다. nominal1500km² seed 목표는 강제 최대 parcel 면적이 아니다.

fragment1278component/3435pixel을 원 candidate1과 대조했다. 이동 pixel과 계수한4neighbor edge는 kind/actualurban/eligibleclass가 같고, <=8 비core 조각→>8 stable target, sharededge→size→ordinal 순서가 맞다. target 전체의 다른 위치에 작은 미분리 urban이 있는 것은 새 homogeneous-target 요구로 거부하지 않았다. dense ID감소1324와 raw movedpixel3435는 다른 단위다. counter11840=427 small-neighbour-only+11413 external-same-surface-neighbour 없음, mixed38별도이며 자연섬 계수가 아니다. `[0×9,1,0,1]` 및 class 고립 fixture를 직접 확인했다. pre-normalization rawseed 전수 저장본이 없어 역사 rawseed 전체 복원을 주장하지 않는다.

IrishSea/EnglishChannel/Japan근해/Atlantic window는26/18/64/55 seaID. 같은 zoom45 연안·해협/대양 화면과 실제 선택·원pixel kind, holes·seam을 확인했다. Atlantic wholeprovince p50 약250737.54km², EnglishChannel 약23.21km²에는 원천 tiny가 포함되며 분포 전체와 분리해 읽어야 한다. 상위 전략해역은 미구현이다.

### 원천·현대 자료 한계

고정7 입력은 land5.1.1/lakes5.0.0/rivers5.0.0/populated-place SHP5.1.2/urban4.1.0 및 NOAA ETOPO2022 두 subset이다. SHA·ZIP CRC·개별 member SHA·PRJ/WGS84·SHP lon/lat axis·embedded VERSION·TIFF 형식/extent/metre/PixelIsArea를 직접 대조했다. CRC/full member 검사는 DBF rawbytes를 decompress/read하지만 속성 파싱·인구·LandScan 채택은 없다. generator는 SHP 지리와 고정 DEM만 읽으며 원작 지도·ID CSV·색/경계/수치표가 입력에 없다. runtime 외부 API/tile/CDN/font 요청도 없다.

Urban ZIP embedded4.1.0와 [공식 웹4.0.0](https://www.naturalearthdata.com/downloads/10m-cultural-vectors/10m-urban-area/) 표기를 구분했다. MODIS2002–2003/1km·1:10M 일반화 자료이며1936 경제/인구/승점에 쓰이지 않는다. [Natural Earth public domain](https://www.naturalearthdata.com/about/terms-of-use/)과 [NOAA ETOPO metadata](https://www.ncei.noaa.gov/access/metadata/landing-page/bin/iso?id=gov.noaa.ngdc.mgg.dem%3Aetopo_2022)의 CC0/CRS/EGM2008 조건을 재확인하고 raw HTML·공식 VERSION 응답을 보존했다. 모든 고정 source 네트워크 재취득은 하지 않았다.

ProvGen planning 연구/receipt와 제품 비교문서를 대조했다. ZIP source/LICENSE 없음·EXE/DLL 미실행/미디컴파일·BMP 미열람/미추출/미사용 한계가 유지된다. 입력 역할·해안거리·크기/산악폭 설명을 내부 Voronoi/watershed/Poisson/Lloyd/growth/merge/determinism 확인으로 확대하지 않았다. 예시값·palette·canonical format을 채택하지 않았고 optional 모더 density image 입력은 미구현 후보다.

### 실제 화면·GPU·3D·HUD

자기 localhost/PID/profile의 headless Chrome만 사용했다. 실제 WebGPU NVIDIA Turing/texture8192/WGSL17782와 WebGL2 ANGLE GTX1660Ti D3D11/texture16384/GLSL29143 및 렌더 frame을 관측했다. 세계·지역·도시·sea PNG, KO/EN, geography/province/terrain·borders·3D toggle·zoom/pan·hover/select/clear·850×650·1280 이상·DPR2를 직접 검사했다. 검은/미렌더 canvas나 foreground 대체는 없다.

850 KO rail 마지막버튼504<=536, EN491<=536, legend와 region overlap없음. source1pixel ID15287/15621은 zoom60/1600viewport의11.015625screenpixel 표본에서 실제 pick이 일치했다. 모든tiny의 모든 축척 선택 편의성을 보장하지 않는다.

핵심 broad capture page/console/HTTP error0. raw failedRequests12는 actual signal-aborted 취소12, unexplained0이다. 카메라 비교 각 raw2는 취소2다. 원 failedRequest를 단순0으로 쓰지 않았다. broad own HMR WS6개와 gameplay/unknown WS를 구분했으며 gameplay/unknown0이다. 사용자 desktop/CUA/tabs/focus/global input/BPhelper·타 PID는 조작하지 않았다. 자기 PID inventory와 browser native0/자기 Vite 종료를 기록했다. Vite termination native1은 의도한 own-child 종료다. parent browser PeakWorkingSet은 renderer/GPU subprocess·JS heap·total memory 측정이 아니다.

GLB 원본/publiccopy/Node recipe whole bytes가 같고22mesh(army6/air8/navy8),160/144/152triangles·6sample912다. finite/CCW/normals/material/metre/+Yup/+Zforward/groundpivot/3axisbbox를 검사했다. 실제 scene/light/depth/camera의 on-off17906pixel 차이, behind-off0, cleanup scene1→0·geometry22/material10 dispose를 확인했다. raw GLB URI/images/hash/path/header/emptygeometry 및 abort를 실제 module이 거부했다. PNG billboard가 아니다.

새 source fresh2 vs Git446 데이터의 같은 current renderer/카메라 **21 PNG 쌍은 모두 전체 RGB와 지도 중심566400pixel이 동일**하다. 원 producer의20전체exact/21mapcrop exact와 Dublin 하단80×32 버튼 차이는 별도 역사 증거다. 당시 hover/focus 미기록으로 그 차이의 원인을 확정하지 않는다. Git446 데이터 비교를 전체 old source tree 재실행이라고 부르지 않는다.

HUD 국가·날짜·경제·자원·8rail·실제unitstate는 미연결/disabled이고 새 gameplay command가 없다.

## 결정론

| 구분 | 해시1 | 해시2 |
|---|---|---|
| M0 | `b039d35666b77fc2` | `b039d35666b77fc2` |
| M1 | `b595dc2a1e5b4f8c` | `b595dc2a1e5b4f8c` |

workspace 저장·재개 및 새 native process M1 resume 통과. 세 fresh의 네 산출물 exact bytes 비교도 통과했다. producer 산출물을 새 생성으로 복사하거나 기존 profiling 결과를 fresh bake로 사용하지 않았다.

## 금지 사항 점검

| 항목 | 결과 |
|---|---|
| sim f32/f64·HashMap·thread_rng/SystemTime | sim/AI 검색0, 보호 경로 unchanged |
| 수치·화면 문자열 | 제작defines·기존 ko/en Fluent, 기존 prefix 보존 |
| client 게임 규칙·미결정 규칙 | 없음, display adapter의 국가/주/권위 필드 미연결 |
| 골든·기존 assertion/config/timeout/retry 변경 | 없음 |
| 클린룸·license·ASSETS | 독립 고정 source, 원 prefix와 union 보존·release check 통과 |
| 새 앱/세션·commit·main/다른 worktree 변경·설정 우회 | 없음 |

## 작업 로그 증거와 실제 결과 대조

원 producer ZIP50,409,960B/SHA `348727bdbe6637f375ef51d62e0dc96745c178eed6d6b77b901f7e18a8d6f036`의556member/554artifact/326criticalsource를 직접 대조했다. actual completed final을 parent API 원 receipt와 exact text 대조했다. snapshot/check-out의 EOL 차이3개는 rawSHA와 canonical equality를 각각 기록했으며, **자기 최초 rawindex 불변성의 면제로 쓰지 않는다.** latest planning12개는 source HEAD와 분리했다.

역사 P05-1/2/3 원 ZIP/MF payload533/1121/861개와 CRC·SHA를 다시 대조했다. a28 F01/F02, 2d920 F03/F04, exact446 F05(174.156native0 뒤180.078/180.188native1)를 유지한다. ab64 품질 미달·dirty 후보·P06-3/4 초기 기하 기대값 오류·counter RED·producer palette139PASS1timeout와 별도140PASS·공간 부족 preparation 실패를 지우거나 새 PASS로 소급 변경하지 않았다.

자기 auxiliary 오류도 보존했다: 최초 바깥 wrapper cwd, custody의 raw EOL 동일 가정·actual-final phase명 오인, cp949 decoding, Tokyo 내부parcel에 nonurban 직접 인접을 요구한 잘못된 추가 조건. ignored helper만 수정했고 원 실패 stdout/stderr/native를 유지했다. 원 제품 테스트·골든 단언을 변경하지 않았다.

부모·자기 최초·중간·마지막 full tracked10017의 각 SHA/목록/HEAD/semantic index/raw index/diff/cached/status가 모두 같다. raw index SHA:
`d30e2af98022fa3b6f70c5a0add1e3ba484c3676aa8a5fd47f4fc03c498bc6db`.

자기 원 stdout/PNG/입력metadata/검사 helper·source provenance를 [SHA256SUMS.json](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify4/target/evidence/WORLD-PREVIEW-M2-r2-P05-4/SHA256SUMS.json)과 새 [evidence.zip](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify4/target/evidence/WORLD-PREVIEW-M2-r2-P05-4/evidence.zip)에 봉인했다. 정확 ZIP bytes/SHA/CRC/member 수는 [ZIP-IDENTITY.json](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify4/target/evidence/WORLD-PREVIEW-M2-r2-P05-4/ZIP-IDENTITY.json), 봉인 뒤 full/raw 비교는 [POST-SEAL-COMPARISON.json](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify4/target/evidence/WORLD-PREVIEW-M2-r2-P05-4/POST-SEAL-COMPARISON.json)에 따로 기록한다. profile cache와 자기참조 파일만 명시적으로 제외하며 원 ZIP은 덮지 않는다. 실제 앱 final은 이 리포트 전문으로 제출하고 CLI0를 앱 종료로 대신하지 않는다.

## FAIL 항목

| # | 재현·범위 | 기대 | 실제 |
|---|---|---|---|
| 새 P05-4 제품 FAIL | 없음 | 새 정확 source의 한정 계약 통과 | PASS; 보완 전 auxiliary 실패와 역사 FAIL은 위와 같이 보존 |
| 역사 F05 | exact446 bounded bake | 원180초 제한 | 원180.078/180.188native1 유지, 새 source PASS가 소급 취소하지 않음 |

미실행·한계: 모든source 네트워크 재취득, historical rawseed 전수복원, 모든tiny의 전축척 클릭 전수, 전체GPU/renderer/JS memory·foreground 성능, globalDEM·정밀능선/고개·강network, 사용자 외관 승인, 동일 HEAD main CI·WP/마일스톤 전체·1936게임 기능. 이 미리보기 범위에 결정 필요는 없다.
