# 검증 리포트 — WORLD-PREVIEW-M2-r2-P05-3

| 항목 | 값 |
|---|---|
| 판정 | **FAIL — 유효한 독립 검증** |
| 검증자 | 구현 비참여 새 Codex 앱 검증 세션 |
| 커밋 | detached `446540f9755f6d381f789192f3576a8cc17aadd2` |
| 검증 일시 | 2026-10-07 21:18~21:48 KST |
| worktree | `E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify3` |
| 범위 | 세계지도·coverage·시가지·해상·HUD·실제3D 미리보기 |

**재생성 시간 예산을 두 번 초과하여 FAIL이다.** 데이터·화면·기존 회귀와 F01~F04 검사는 통과했다. WP-32/45 전체·M2/M5·1936 콘텐츠·게임기능·사용자 외관 승인·main CI 완료를 판정하지 않는다.

## 직접 실행한 명령

모든 명령은 지정 worktree와 `GIT_OPTIONAL_LOCKS=0`에서 실행했다. 실제 argv/PID/cwd/native exit/stdout/stderr는 [증거 루트](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify3/target/evidence/WORLD-PREVIEW-M2-r2-P05-3)에 보존했다.

| 명령·검사 | native exit | 결과 |
|---|---:|---|
| npm ci / typecheck / build / licenses | 각 0 | 통과 |
| npm client test | 0 | **140 PASS** |
| map Python tests | 0 | **26 PASS·1 skip** |
| docs / assets --release / 기존 Fluent checker | 각 0 | 통과, 기존 경고 보존 |
| Rust fmt / clippy / build / workspace test | 각 0 | **179 PASS**, 저장·재개·generated TS equality 포함 |
| architecture / cargo deny / Testland validate | 각 0 | 통과 |
| M0·M1 365일 seed1 각각 두 번 | 각 0 | 각 해시 일치 |
| bounded bake1 | 0 | **174.156초**, shipped 4파일 bytes 일치 |
| bounded bake2 | **1** | **180.078초**, time budget exceeded |
| 추가 bounded bake3 | **1** | **180.188초**, 같은 실패 |
| 독립 SHP·면적·ID·urban·인접·F03/F04 검사 | 0 | 전수·경계 검사 통과 |
| ignored source mutation 5종·Fluent 키 제거 | 각 1 | 예상 거부, 원천 bytes 보존 |
| GLB 재생성·geometry·fragment 전수 대조 | 0 | 통과 |
| 최소인자 headless Chrome·3D·negative 검사 | 각 0 | 실제 화면·browser native exit0 |
| 원 Chrome 전체 E2E | 0 | **단일 실행 41 PASS** |
| 원 Firefox 전체 E2E | 0 | **단일 실행 41 PASS** |
| 원 WebKit 전체 E2E | 0 | **단일 실행 41 PASS** |
| 봉인·봉인 뒤 full/raw 불변 비교 | 각 0 | 전수 동일 |

Python skip은 과거 ignored DEM 취득경로 부재다. 고정된 두 TIFF는 별도 독립 Pillow decoder와 **모든 픽셀 equality**, finite·extent·620개 대표 고도를 직접 검사했다. skip을 숨기거나 다운로드 fallback으로 바꾸지 않았다.

원 tracked bake/TS writer는 호출하지 않았다. E2E의 원 fixture/assertion/timeout/retry/workers/testMatch를 보존했다. Chrome에는 최소 headless 인자와 width 검사를 위한 cosmetic `hide-scrollbars`만 지정했다. sandbox/security/GPU override와 foregroundfallback은 없다.

초기 executable 검색·UTF-8 읽기·독립 helper의 인자/필드/ref 오류는 원 실패 로그를 남기고 ignored helper만 수정했다. 미실행 명령을 성공으로 적지 않았다.

## REQ별 확인

| REQ·계약 | 직접 검사 | 결과 |
|---|---|---|
| REQ-MAP-01/02/07 일부 | sourcekind·RG8/u16LE·dense ID·4connected·bounds·대표좌표·seam | 데이터 PASS |
| REQ-MAP-04/05 일부 | canvas·pick oracle·hover/select/clear·zoom/pan·모드·resize/DPR | PASS |
| REQ-MAP-10 | fixed7 원천·공식 metadata·CRC·입력 거부·offline bake | 원천 PASS, 반복·예산 **F05** |
| REQ-MAP-11 일부 | 실제 river/DEM·대표 고도·빈 graph fallback | 한정 PASS |
| coverage·major urban·sea/lake | 전체 mask/ID·component/window·실제 선택 | 한정 PASS |
| REQ-LOC-01/03·F01 | 원 Fluent checker·KO/EN·필수 키 제거 거부 | PASS |
| 기존 renderer·F02 | 새 exact446의 세 browser 각각 원전체41 | PASS |
| F03/F04 | valid2×2·noanchor floodfill·65535/65536·실제 client | PASS |
| 실제3D·HUD·REQ-NET-04 | GLB/scene/depth/cleanup·미연결 슬롯·TS equality | 한정 PASS |

### 지도·지역·시가지·해상

- **8192×4096, 24,889ID = land13,882 / sea6,954 / lake4,053.** index67,108,864bytes, 모든 ID4connected, sorted unique adjacency47,554쌍.
- 독립 SHP decoder/scanline raster로 **33,554,432픽셀** sourcekind·urban/major class가 모두 일치했다. 1-based 표시ID와 0-based ordinal을 구분했다.
- 구면 면적은 defines R=6371.0088km와 `R²Δλ(sinφN−sinφS)`로 독립 재계산했다. 최대 반올림 오차 약4.99967e-7km². raster 근사를 측량·국가면적으로 부르지 않았다.

| 분류 | 전체1pixel province | 원천kind 자체1pixel 조각 | <=8pixel province |
|---|---:|---:|---:|
| land | 2769 | 2366 | 5586 |
| sea | 2349 | 2267 | 4125 |
| lake | 1492 | 1492 | 3065 |
| 합계 | **6610** | **6125** | **12776** |

sourcekind보다 추가된1pixel485를 자연 섬 계수로 표현하지 않았다. land/sea/lake 전체 면적 p50은315.89/42.28/33.38km², p90은34950.81/208121.85/370.68km²다. tiny 때문에 중앙값만으로 품질을 판정하지 않았다.

Ireland의 동일5,789pixel/약82,360.65km² 연결섬은 ab64 원본 **1ID6809 → 현재17ID**다. 현재13개>=9pixel, tiny1/2/2/8 네 개, largestshare17.17%. GB68ID/26.40%, Hokkaido16/19.76%, Shikoku10/45.25%. Honshu/Kyushu는 원천상 같은173ID 연결 component다. 큰 land75개 공통 면적 최소 quota와 landseed6000을 확인했고 Ireland 특례·임의 행정경계·단절 merge가 없다.

Eurasia/Africa와 North/South America의 source 연결 component를 지리 crop와 구분했다. Europe/Africa/EastAsia/NorthAmerica/SouthAmerica crop의 ID는2195/1575/1809/3189/1254이며 완전한 국가·대륙 수치가 아니다. 전체 면적·큰셀 편중·도시점 coverage·actualIDs는 [coverage.json](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify3/target/evidence/WORLD-PREVIEW-M2-r2-P05-3/coverage.json)에 있다.

시가지는 **polygon∩land**, >=100km² major **1846component**의 실제 province 경계를 전수 검사했다. 작은 footprint는 미분리이며 도시점7342 중 sourceurban 밖4119는 누락으로 남겼다.

| 도시 | 실제 선택ID | province pixel / 근사km² | urban fraction |
|---|---:|---|---:|
| Dublin | 23023 | 24 / 342.17 | 1.0 |
| London | 23080 | 161 / 2392.75 | 1.0 |
| Seoul | 23751 | 136 / 2566.56 | 1.0 |
| Tokyo | 23828 | 80 / 1550.20 | 1.0 |

같은 zoom60 PNG·source point·원pixel oracle·주변 nonurban을 대조했다. 1500km²는 nominal seed 목표이며 최대 parcel 규칙이 아니다. 도시점 증가를 footprint 분리로 대신하지 않았다.

fragment1278component/3435pixel의 <=8 조각·>8 안정target·조건 일치4neighbor edge·sharededge→size→ordinal을 원 candidate1과 직접 대조했다. dense ID감소1324는 다른 단위다. 정정counter11840=427 작은same-surface 이웃만 존재+11413 외부same-surface 이웃없음, mixed38별도. 이를 물리적 섬 계수로 쓰지 않았다. historical pre-normalization rawseed는 저장돼 있지 않아 전수 복원 주장 없이 final recipe와 별도 geometry fixture를 검사했다.

IrishSea/EnglishChannel/Japan근해/Atlantic crop는26/18/64/55 seaID. 동일 축척 연안·해협/대양 PNG와 실제 sea 선택을 확인했다. 모든 sourcekind·hole·seam 인접은 원raster와 같다. **상위 전략해역은 미구현**이며 Kyushu 수로·Caspian 원천 누락을 발명한 경계로 보완하지 않았다.

고정7 source SHA/ZIP CRC/PRJ/embedded VERSION/두 DEM 형식·CRS·axis·메타데이터를 대조했다. CRC는 DBF rawbytes도 읽지만 속성·인구·LandScan 채택은 없다. Urban embedded4.1.0와 [공식 웹4.0.0](https://www.naturalearthdata.com/downloads/10m-cultural-vectors/10m-urban-area/) 차이, MODIS2002–2003/1km·1:10M 일반화를 명시했다. [Natural Earth public domain](https://www.naturalearthdata.com/about/terms-of-use/) 및 [NOAA ETOPO metadata](https://www.ncei.noaa.gov/access/metadata/landing-page/bin/iso?id=gov.noaa.ngdc.mgg.dem%3Aetopo_2022)의 CRS·EGM2008·CC0를 확인했다. 현대 위치·시가지가1936 경제/인구/승점에 쓰이지 않는다.

### 실제 화면·3D·HUD

최소인자 자기 headless Chrome에서 WebGPU NVIDIA Turing/texture8192/WGSL17782, WebGL2 ANGLE GTX1660Ti D3D11/texture16384/GLSL29143과 실제 frames를 관측했다. 세계·지역·4도시·해상 PNG, KO/EN, geography/province/terrain·3D toggle·borders·zoom/pan·hover/select/clear·850×650·1280이상·DPR2를 직접 검사했다.

850 rail 마지막 버튼은 KO504<=536, EN491<=536이고 legend/region overlap없음. ID15287/15621의1pixel 표본은1600viewport/zoom60/**11.015625screenpixel**에서 원pick 일치했다. 일반 축척 선택 편의성 보장은 아니다.

page/console/HTTP오류0. raw failedRequests12 중 actual signal-aborted 취소12, unexplained0. 자기 HMR WS6개와 gameplay/unknown WS를 구분했다. 외부 runtime API/tile/CDN/font없음. 사용자 desktop/CUA/tab/focus/globalinput/BPhelper·다른PID를 건드리지 않았다. 자기PID 정리 확인.

실제3 GLB 원본/publiccopy/Node recipe재생성 bytes 동일, **22mesh/160·144·152triangles, 6샘플912**. finite·CCW·normals·material·metre/+Yup/+Zforward/groundpivot·3axisbbox를 검사했다. 동일 source scene의 on/off17906pixel 차이, 지도뒤 mutation/off차이0. cleanup scene1→0, geometry22/material10 각각dispose. abort·hash/path/header/URI/images/emptygeometry를 실제 module에서 거부했다. PNG billboard가 아니다.

HUD 국가·날짜·경제·자원·8개 rail·실제unitstate는 미연결/disabled다. 전체 GPU/renderer/JS memory 또는 foreground 성능을 측정했다고 하지 않는다.

## 결정론

| 구분 | 해시1 | 해시2 |
|---|---|---|
| M0 | b039d35666b77fc2 | b039d35666b77fc2 |
| M1 | b595dc2a1e5b4f8c | b595dc2a1e5b4f8c |

workspace 저장·재개와 새 native process M1 resume 통과. bake1의 index/provinces/adjacency/metadata는 shipped와 exact동일이다. **bake2/3은 산출하지 못해 독립 반복4SHA 비교를 완료하지 못했다.** producer 후보5/6 일치를 새 검증의 반복 성공으로 대신하지 않았다.

## 금지 사항 점검

sim/AI 금지float·HashMap순회·thread_rng/SystemTime 검색없음. crates/proto/network/save/golden/원E2E 보호경로는 base대비 그대로다. Fluent KO/EN·ASSETS의 기존prefixbytes 보존. 수치는 제작defines, 문구는 기존Fluent경로다. 미결정 게임규칙·원작 경계/IDCSV/색/수치표 입력없음.

ProvGen은 planning README/Settings 연구 범위만 대조했다. EXE/DLL 미실행·미디컴파일·BMP 미열람/미추출/미사용. 내부 알고리즘 실체 확인·예시값/palette/canonicalformat 채택 주장이 없다. optional밀도image importer는 미구현 후보다. 새 세션·commit·mainwrite·설정 우회없음.

## 작업 로그 증거와 실제 결과 대조

원 producerZIP107651219bytes/SHA `fe53808374917360550ddf6bff6d2496b9519a99d2f9274ee069103a7fa4dab0`, **673member/671artifact/324criticalsource 전수일치**. 원snapshot과 checkout의 UTF-8 EOL차이3개는 rawSHA와 canonical내용을 각각 보존했다. planning10개는 제품source와 분리했다. 과거 a28/2d920 FAIL·ab64 품질부족·dirty후보·원RED/native실패를 보존했고 소급PASS로 바꾸지 않았다.

부모·최초·중간·마지막·봉인뒤 **tracked10015 각SHA/목록/HEAD/semantic·rawindex/diff/cached/status 전부동일**. rawindexSHA:
`b6dfe05f2c0d0bf3b7d72a2a3a30978088f59567338b6e6b0641613dba9ce07d`.

[전체 리포트](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify3/target/evidence/WORLD-PREVIEW-M2-r2-P05-3/VERIFY_REPORT.md), [ZIP identity](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify3/target/evidence/WORLD-PREVIEW-M2-r2-P05-3/ZIP-IDENTITY.json), [봉인뒤 비교](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify3/target/evidence/WORLD-PREVIEW-M2-r2-P05-3/POST-SEAL-COMPARISON.json).

봉인 ZIP **155621367bytes/862member**, CRC·모든member SHA검증:
`7790c6b6043de851602e3d737f8e0d8d265935572cb981faa36d8cdd7276ac7e`.

## FAIL 항목

| # | 재현 | 기대 | 실제 |
|---|---|---|---|
| **F05** | exact446에서 `tools/maps/bounded_bake.py`, 같은 fixed입력·신규ignored out/receipt | 두 번180초/3GiB/64MiB 안에 종료·4SHA일치 | 첫174.156s/native0, 두 번째180.078s/native1, 추가180.188s/native1·time budget exceeded |

peakCommit은 각각2639523840/2639208448/2640293888bytes로3GiB 안이다. 감독의 ownPID 전기간 peak counters와 samples를 보존했다. 일부 기본회귀/E2E/독립화면 병행 부하를 명시했으며 환경 전체 성능 보장으로 확대하지 않는다. 실패를 덮거나 limit/해상도/timeout/단언을 완화하지 않았다.

미실행·한계: 모든source 네트워크 재취득, historical rawseed 전수복원, 모든tiny의screenclick 전수, 전체GPU/JS memory, globalDEM·정확능선/고개·강network, 사용자 foreground 외관 승인, mainCI·전체WP/마일스톤·1936게임기능. 이 결과는 **FAIL**이며 CLI0나 봉인 성공으로 대체하지 않는다.
