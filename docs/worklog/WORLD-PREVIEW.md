# WORLD-PREVIEW 작업 로그 — 실제 세계 지도와 메인 HUD

| 항목 | 값 |
|---|---|
| 상태 | 독립 검증 대기; 자체 실행 증거 제출 |
| 담당 | 구현 세션 |
| 브랜치 | codex/world-province-preview-m2r2 |
| 대상 REQ | REQ-MAP-01/02/04/07/10 일부. REQ-MAP-03/11 및 WP-32/45 전체는 미완료 |
| 선행 WP | 검증된 WP-07/08 renderer; base ea63081e0b79369227c99384922db80c6ef09fb1 |

이 문서의 초기 테스트·누락 서술은 첫 e209/a28 체크포인트 역사이며, 현재 제출 상태는 마지막 최종 증거 항목을 따른다.

## 계획

최초 schema/ADR 작성 후 fixed land/lake mask·구면 독립 분할을 bake하고 기존 renderer의 preview-only display adapter에 연결한다. 실제 전세계와 지역 미리보기를 먼저 제출하고 후속 3D 샘플·DEM 취득·세부 검사를 진행한다. 원 nation/state/sim/network/Testland 변경 없음. 전용 worktree에서만 명령 실행. 부모 외 새 세션 생성 없음.

## 테스트 우선 기록

| REQ | 테스트 이름 | 구현 전 결과 | 구현 후 결과 |
|---|---|---|---|
| REQ-MAP-10 | source missing/hash mismatch/path escape | module missing exit1 원로그 python-red.log | 3 tests PASS |
| REQ-MAP-01/02/07 | connectivity/island/seam adjacency, polygon hole/coordinates, u16LE | 동일 원실패 | 3 tests PASS |
| REQ-MAP-04/10 | baked SHA/schema/ref/IDs/display has no authority | model missing exit1 client-red.log | 전체 client132 tests PASS |

## 실행한 검증

모든 원로그는 이 WT `target/evidence/WORLD-PREVIEW/`에 보존.

| 명령 | 종료 코드 | 출력 발췌 / 로그 경로 |
|---|---|---|
| python -m unittest discover -s tools/maps -p test_world_preview.py | 0 | python-green.log, 6 tests |
| npm --prefix client ci | 0 | npm-ci.log, 103 packages, 0 vulnerabilities |
| npm --prefix client run build | 0 | build.log; 기존 large chunk warning 유지 |
| npm --prefix client test | 1→0 | npm-test.log: cargo PATH 미발견. 프로세스 PATH에 기존 C:/Users/user/.cargo/bin 추가 후 npm-test-path.log:132 PASS. 전역 설정 미변경 |
| python tools/check_assets.py | 0 | assets-first.log, 0 errors |
| python tools/check_docs.py | 0 | docs-first.log, 0 errors/warnings |
| node tools/maps/capture_preview.mjs | 1→0 | 처음 Vite root 미지정으로 404 원실패 capture-first.log. own PID9940 identity 확인 후 종료하고 positional client root로 재기동. capture-full-width.log exit0 |

## 증거

- E1: land/lakes source-manifest 및 원 ZIP SHA, defines/metadata. 최초21097 bake에서 geometry 연결성을 보존하며 seed 수 조정 후14510 실제 ID. 조정은 게임 콘텐츠 변경이 아니다.
- E2: `target/evidence/WORLD-PREVIEW/world.png`, `europe.png`: 격리 headless Chrome 1600×1000 실제 rendered page. WebGPU/NVIDIA Turing, WGSL10539, frames14/10. 오류 없음. 실제 interactive foreground UI 검증/성능 벤치라고 주장하지 않는다.
- E3: browser-process.json 및 browser-capture.json: own headless PID/argv/cwd/profile, console/requests/frame/adapter/camera. 사용자 desktop/tab/focus/globalinput 조작 없음. 최신 사용자 지시 이전 CUA inventory read-only 1회 후 중단. 모든 screenshot은 ownpage만.
- E4: server-process.json, vite.stdout/stderr.log: own node Vite local127.0.0.1:4317, WT cwd, port/argv/PID. 실행: `node client/node_modules/vite/bin/vite.js client --host 127.0.0.1 --port 4317 --strictPort`; URL `http://127.0.0.1:4317/?world-preview=1`.
- 초기 generic shell CSS가 main960px/section 스타일을 preview에 적용하는 실제 캡처에서 발견, preview-local override 후1600 full viewport 재촬영. 최종 첫 캡처는 override 이후만.

## ADR

- [ADR-3201](../adr/ADR-3201-world-preview.md), [계약](../plans/WORLD-PREVIEW-schema.md).

## 원작 대비 차별화 초안 (01 §11에 넣을 행)

- 원작 경계·UI·국기 없이 독립 generalized geography/구면 분할, 지리 provenance와 실제 누락 표시. HUD는 후속 권위 표시 슬롯만 제공하며 게임 기능 미연결을 표시한다.

## 결정 필요

없음. 국가/역사/주/게임 수치 승인은 이번 시각 요청에 포함되지 않아 구현하지 않았다.

## 범위 밖 발견

- 원 large renderer bundle 경고. 기존 renderer palettes의 반복 linear find 비용이 약180ms/update 관측. 기존 renderer 강도 변경 없이 우선 재사용.

## 못 한 부분과 이유

- 첫 체크포인트: DEM·능선·하천·도시 density 미반영; REQ-MAP-11 전체 미완료. source resolution 이하 섬/호수는 미보존 가능.
- 3D unit leaf asset 부모 인계 대기; 화면 unit slot은 아직미연결. 최초2DPNG는 등록/copy하지 않는다. 이후 실제3D sample overlay 요청을 별도 후속 체크포인트에서 연결한다.
- zoom/pan/select/border/모든region/resize/WebGL2·Rustfmt/clippy/workspace/hash2 및 반복bake 종합검사는 후속 진행. 기존 독립PASS/mainCI/WP 전체/M5 완료를 선언하지 않는다.

## 수정 기록 (P-06)

Vite root/cargo PATH/preview CSS 문제 원실패 보존 후 수정.

### 첫 실제 3D 샘플 후속

부모가 exact producer19cf3cf6dab7efbcaf1f3abf0f2b53d14496324a를 인계한 뒤 procedural units3d leaf10개만 복사·SHA 검증·master append. 원AI PNG는 인수하지 않았다. army/air/navy는 embedded geometry/opaque PBR, texture/image0. 육군160/공군144/해군152 triangles를 2개 지역샘플씩 배치하여 총6샘플912 triangles. 위치/heading/artscale/lighting은 units/defines.toml이고 미래군대 state가 아니다. 실제 Three sceneExtension hook, depth-tested mesh, orthographic camera z1000, ambient/directional lighting. 기본 renderer의 hook없는 App/Testland 경로 그대로. shader/pipeline assert/fallback/input/resize 보존. source hashes·recipe/generator·metadata·copy identity는 asset metadata 및 ASSETS에 기록.

units3d.test.ts는 remote buffer/sprite/empty/nonGLB rejection RED(modulemissing exit1)→PASS. 이후 실제 GLB shape/bounds 검사와 조작/회귀 검사를 계속한다. npm typecheck exit0. capture-3d-first.log exit0: world/europe WebGPU NVIDIA Turing, samples6/triangles912. first404 원출력은 e209 봉인에서 그대로 보존. 새 diagnostic은 console location으로 /favicon.ico 404 확인. preview-only own favicon 추가 후 실제 새페이지 consoleErrors=[]/badresponses=[]; devStrictMode 첫 effect cleanup으로 metadata/units fetch net::ERR_ABORTED는 개발환경 의도 취소이며 오류와 분리한다. 첫 e209 화면은 pageerrors=[]였지만 resource4041이 있었고 전체console0 주장은 하지 않는다.

Rust fmt/clippy/workspace-test/365dayshash×2/validateTestland exit0. hash b039d35666b77fc2×2. bake repeat의 index/provinces/adjacency/metadata 4파일 exact SHA 모두 일치. known coordinates 및14510 ID 전수4connected 검사 PASS(추가 post-bake 검사, RED 선행으로 기록하지 않음).

NOAA 공식 GridExtract datasets.json template로 30s/4MiB 예산의 Himalaya [74,25,95,35], F32 TIFF1008×480/75arcsec(원60arcsec) 취득: 1979963bytes/3.72s/SHA14e07b92aa2e8f4bcc09de6c8ead73a5b098fc6ff74a0ce6971ba78261141452. GeoTIFF local strict LZW reader를 RED(functionmissing)→2 PASS로 시험. EGM2008metres/EPSG4326/PixelIsArea/finite/noNoData범위 확인. 자료는 아직 target/evidence에 있으며 현 checkpoint 지도에 미적용. global DEM/능선/하천/도시 밀도 구현을 뜻하지 않는다.

## 통합 기록 (P-07)

구현 브랜치 commit까지만. main merge/push/tag/public hosting 없음.

## 최종 제출 상태와 증거

- 현 지도 source는 land5.1.1/lakes5.0.0/rivers5.0.0 + 고정 NOAA ETOPO2022 v1 60s surface Himalaya75arcsec subset이다. 4096×2048/PlateCarrée/8neighbor同kind geodesic allocation/4connected ID. 실제13,413=land7,103/sea3,641/lake2,669, valid대표고도147개. river거리/DEM interior slope 밀도, rivercrossing/실제높이차·localrelief 경계cost. noise굴곡 없음. 3Darmy/air/navy6샘플912triangles, ownmaterial/depth/light. 실제게임군대가아니다.
- Source/version/terms/SHA는 source-manifest/ASSETS, recipe는 tools/maps 및 units3d generate files. 같은입력/options/toolversions의 final2bake에서 index/provinces/adjacency/metadata4SHA 전부일치. 3D3원GLB인수SHA exactproducer19cf3cf로고정. ASSETS의base ea63081 prefixbytes 보존; originalFluentko/en prefixbytes 보존하고preview-*만append. 기존game/network/proto/sim/checker/Testland/골든변경 없음.
- 자체 검사는 Python11PASS, client135PASS, build/typecheck0, npm103deps licenses0, docs/assets0. 원Rustfmt/clippy/workspace0; M0hash365x2 b039d35666b77fc2, M1 `--pack data/packs/testland --scenario m1`365x2 b595dc2a1e5b4f8c. proto generator exit0/protocol.ts gitblob84d88d8a39a8b2d4ce804fe2ffdfb7ae834d84cd 전후동일. Rust/프로토콜코드는base동일이므로해당회귀logs 적용범위를구분한다.
- 기존M1Playwright원3browser전체123PASS/2.8m(exit0), defaultscenehook없는renderer/backend/fallback/pipeline/resize/DPR/redirect/invalidmetadata/국가패널검사. 처음rootcwd의config.resolve('../target') 실패m1-chromium.log 보존 후 정확clientcwd에서같은원검사를실행했다. 원테스트/assert/fixture/baseline을수정하지않았다.
- 새미리보기 headlessChrome ownpage15captures/fullactions exit0: world/europe/Korea/Himalaya terrain/Americas, actualhoverpick3923, wheel6→10.173/pan변화, borderoffcamera동일, provincepalette색변화, samplesoff0/on6,850×650/1600×1000/1280×800resize,ko/en, explicitWebGL2. actualWebGPU NVIDIA Turing, WebGL2 ANGLE NVIDIA GTX1660Ti Direct3D11. 페이지예외/consoleErrors/resource400+0, externalruntime요청0. ReactStrictMode원effect취소ERR_ABORTED는진짜오류와분리했다. fullrun의EN visibletext/aria-label언어selector timeout 원실패capture-full.log/browser-full-selector-failure.json/full-selector-failure.png 보존 후correctaccessible selector로재검사. 실사용자foreground브라우저/desktop성능벤치라고주장하지않는다.
- 독립a28locale검수후 동일문제를 tools/check_localisation.py native1로 직접재현(localisation-red.log/uses). previewTSdict는rootFluentAST계약과불일치했다. checker나테스트를수정하지않고 기존ko/en.ftl에preview-*키만append 및 PreviewHud/WorldPreview의literal/dynamic/runtime 모두같은Fluenttranslator로정렬. 각키의ko/en실제번역회귀검사추가. localisation-green.log native0(기존미사용서버메시지 warnings는보존), client135PASS/build0/actual fullpage캡처로재검사. a28독립판정은새HEAD로전용하지않는다.
- 형상전후비교는같은renderer/HUD로a28고정baked3파일을git show에서읽어격리page에제공하는comparison-a28-inputs와최종자료comparison-final을구분한다. 양쪽Europezoom6/Koreazoom23/Himalayazoom12 동일카메라·지리모드·viewport1600×1000. 과거inputs상태비교이며a28전체코드의재캡처라고표시하지않는다. 기존checkpoint-e209d0e/checkpoint-a28f49c 원exactHEAD/원PNG/원logs는그대로봉인보존.

### 현재 미완료와 한계

HUD의국가/날짜/경제/패널명령·실제unitstate미연결. 게임기능/정치경계/역사1936콘텐츠/주연결없음. GlobalDEM·도시density·정확능선/고개/강network검수없음. Himalaya terrain은대표cell고도색표본으로continuousDEMoverlay가아니다. 원천10m축척및4096 raster약0.0879°때문에coast/zoom계단·작은섬/호수/좁은통로누락이있고한국지역일부cells는거칠다. 기본NaturalEarthlakes에Caspianpolygon없어현재sea분류로남음을실제source전수조회로확인했다. lake-hole우선분류수정은실제원천에있는호수에만적용되며Caspian을복구했다는주장을철회·부모에정정했다. 원game경계tracing/파일추출/수치표/UI/국기/아이콘복사없음. Detailedterrain/editorimage포맷은의견질문에그쳐추가계약을만들지않았다.

### 봉인과 인수

최종cleancommit후 `target/evidence/WORLD-PREVIEW/final/`에actualpagePNGs, 실행argv/PID/cwd/nativeexit/rendereradapter/frames/console/network, source ZIP/TIFF·generatedfiles 및SHA,trackedlist/diff/status·identityJSON를봉인한다. 정확커밋/URL/실행법/미완료를부모에전송한다. ownVite127.0.0.1:4317/PID33284를사용자의로컬검토를위해유지한다. main병합/푸시/태그/공개서비스없음. 독립P05PASS/mainCI/WP-32·45/M5완료판정없음.

### 최종 인정 브라우저 실행 인자 정정

부모의a28독립검수후속에따라최종페이지증거는Chromium sandbox/가속·보안 기본값을보존하는최소인자로재생성한다. capture_preview의초기Playwrightdefaultargs에는--no-sandbox 및feature switches가있어앞선시도와PNG/JSON은default-history에보존하고최종인정GPU증거에서제외한다. 최신helper는 ignoreDefaultArgs:true/chromiumSandbox:true, 별도ownprofile, exact --headless/--user-data-dir/--remote-debugging-pipe/--no-startup-window만지정하고BrowserServer도127.0.0.1에만bind한다. OS/브라우저전역설정·사용자기존profile/focus/input변경없음. 최소인자smoke의실제childargv4옵션/Chrome154/PID/profile/cwd/nativeexit0 및realWebGPU/console0을직접관측했다. 최종cleanHEAD에서full/전후geometry비교캡처를이방식으로다시얻으며각JSON에rendererHead/cleanstatus/비교mapSHA·nativeexit를기록한다. 기존M1Playwright123PASS는기능회귀기록이며최소인자최종제품GPU증거와구분한다.

## Quality P06-2 first checkpoint — implementation, independent verification pending

Scope: own preview tools/source/generation/renderer presentation/scoped CSS/append-onlypreviewFluent/documents. Original GLBs/recipes/proto/game/network/testfixtures/checkers remain unchanged. New evidence: target/evidence/WORLD-PREVIEW-quality-P06-2; old2d920final remains immutable.

Tests-first: quality-red.log/quality-edgeless-red.log→quality-green.log; budget-red.log(native1missinghelper)→budget-green.log(native0); palette-red.log→palette-green.log; presets-red.log(native1missingcameraaccepted)→presets-green.log(native0six); coast-kind-red.log(native1actualmutablelabelreuse)→coast-kind-green.log(native1firstfixrejectsvalidfixture)→coast-kind-green-2.log(native0seven). Original fixture/assert/timeout/retry/baseline untouched. Prototype1/2/3/4 and proof-draft excluded from final accepted data. Initial command quoting error and transient TS typing errors are attempt records, not successful checks.

Accepted first data candidate5:8192×4096,20,366IDs=land10,233/sea6,080/lake4,053;35,753adjacency pairs;467actualrepresentativeDEMheights. Fullsourceclassification/allpixelmatch/everyID4connected/bbox/pixels/representativecoordinates/elevation/seamadjacency/source6SHA directcheck native0(candidate-5-validation.json). Index67,108,864bytes/SHA cb35897724820813cd66a8f4d13fc4e682e84b21e4cb544686e1f04f7d07628b. Fullboundedbake84.766seconds/peakWS1,637,036,032/peakcommit2,637,049,856bytes native0,within180s/3GiB. Candidate6same-source repeat runs separately.

Korea geographicwindow[121.9921875,31.9921875,132.01171875,42.978515625] has83modernpointpositions(also neighboringChina/Japan),160newseeds/356existingedgeanchors. LandIDsinwindowbefore62/after287 includeislands/fragments/intersections; medians in validation are not equal-area wholeprovince statistics or proof of uniform density. Europe413points/80seeds;Himalaya114points/60seeds. Citypositions/actualriverdistance/validinteriorDEMslope weight seeds; original kind-geodesic cost unchanged exceptregionalphysicalextent. Independent rectangle edgesretainexistingIDs; cornerwrapforregionalgraph disabled. No political boundaries/history/ownership/population adopted. Immutablecoarselabelread fixesfinecoastclassassignment.

Known limits: moderncitysample sparse,globalDEM/precisemountainpasses/rivernetworkaccuracy incomplete,coastpixelstepsstillvisibleatzoom,narrowpassage/subpixelislandlimits,Caspianstillsea. HUD/date/economy/nation/state/editor/gameunits disconnected. ActualGLBarmy/air/navysixsamples912triangles remain sameassetbytes. No independentPASS/mainCI/WP32/45/M5completion claim.

Pre-checkpoint unchangedRustfmt/clippy/workspace native0;client139PASS/buildnative0(before staticdata replacement); required fullchecks and cleanhead actualPNG follow. Existing bundle-size warning retained.

### 첫체크포인트 후 실제조작과 GPU 업로드 관측

76f7104cleanactualcapture checkpoint-first native0. checkpoint-full native0:zoom6→10.173/pan/selection20236/borderscamera보존/allpresets/koen/GLBtoggle6→0→6/WebGPU8192/WebGL2ANGLE16384/850×650/1280×720/1280×800. 850rail bounds124..536,lastarmy439..504,scrollHeight561/scrollTop151,documentwidth850exact. RawERR_ABORTED4는동일URL의actualsignalabortedcanceltrace4와대조,unexplained0/HTTP400+0/consoleerrors0/pageexceptions0; rawfailures를없다고표현하지않는다. BrowserparentPeakWS171,032,576bytes는renderer/GPUchild/JSheap합계가아니다. JSindex원+decode최소128MiB와GPU RG8index64MiB/lookup추가를input기하로산정,실제전체GPUmemory측정이라고주장하지않는다.

Palette-modechange실제업로드1275.8ms:O(n)CPUadapter지만기존GPUAPI가바뀐20ktexel마다onepixeltexture를만들었다. Previewadapter가있고변경수가lookup한행보다많으면256²전체색표를한번copy하는표시경로만추가했다. 기본App/작은incremental path는기존그대로. palette-upload-full은동일4outputs/동일camera/native0,actualupdate5.6ms,육해/3D/ID/selection값변경없음. 이값은단일headless실측이며성능보증이나벤치평균이아니다. Beforeafterworld/europe/KoreaWebGL2/scrollrailPNGs4개는exactSHA동일. province/terrainPNG부분차이는분석해별도기록한다. 원testfixture/assert/timeout/retry/checker동일.

### 최종 자체회귀 기록 — 독립 판정 아님

원M1Chrome41tests 단일전체실행native0/41PASS/0skip/0flaky/0unexpected/43.9s(m1-original-full.log/.json/.exit/m1-native-summary.json). config는ignored externalm1-minimal.config.ts로originalconfig/testdir를import/참조,올바른clientcwd/own19435webserver. originalasserts/fixtures/timeout/retry보존. 모든actualchildlaunchlines는headless/hide-scrollbars/automaticownprofile/pipe/no-startup-window만,security/GPUswitch없음. 시작시optionalpaletteupload가dirty였으므로m1-tested-source.json의actualproductSHA를최종커밋과대조한다. HEAD만으로검수범위를허위확장하지않는다.

Finalclient139PASS/build/typecheck0/103depslicensechecker0;Python18PASS;docs/assets/localisation0(기존unusedwarnings유지). Rustfmt/clippy/workspace0;M0hashb039d35666b77fc2×2/M1hashb595dc2a1e5b4f8c×2/native0. 원protocol/crates/gamepack/golden/network/checkers/GLBs추적diffempty,ko/enoldprefix/masterea63081prefix보존(protected-paths.json). 이검사는구현자의자체증거이며독립검증PASS/mainCI녹색/마일스톤완료를선언하지않는다.

PNG원픽셀decode대조:province-colors8885changedpixels는toolbar579,113..887,149안에만,terrain2448pixels는Himalayaregionbutton832,897..911,928안에만있다. CSSbutton150mstransition중capture시점차이이며그외map/data/selection/meshpixels동일. 관측renderupdate1,275.8→5.6ms는단일관측,총브라우저/GPUmemory나처리시간성능보증아님. BrowserparentPeakWS166,268,928bytes만직접조회,renderer/GPUchildheap총합아님.

현재scope내사용자결정필요없음. generalized8Kcoast확대계단/작은섬·통로누락가능/globalDEM/정확능선·고개·강network/역사도시·state·nation·economy·gameunit미연결.8Kunsupportedhardware는원capability실패를보고하고해상도fallback하지않는다. 공개hosting/mainmerge/push/tag/CUA/globalinput없음. ownVite127.0.0.1:4317PID33284유지.

최종cleanHEAD실제fullPNG/동일카메라2d920baked입력before/finalafter와원실패/native로그/ZIP6입력(4ZIP·2TIFF)/outputs4/criticalfiles를newfinalidentity로봉인해부모에인계한다. oldtarget/evidence/WORLD-PREVIEW/final/identity봉인은변경하지않는다.

### 작은 화면 유닛 설명과 지역 버튼 겹침 수정

최종시도bf261a8의850×650실제PNG에서legend가worldbutton일부를덮었다. owncapture overlapassert추가후legend-overlap-rednative1실패보존. 1150px이하previewCSS에서legendbottom을114px로옮긴뒤legend-overlap-greennative0/전조작통과. 850regioncontrols와legendbounds가서로겹치지않고rail군버튼끝까지scroll접근가능. 기존App/CSS/원테스트/assert/timeout/retry변경없음. 데이터/GLB/renderer/productTSbytes는bf261a8와동일하며ownpreviewCSS와capturehelper만추가변경. 이전bf261a8final시도captures는이력으로보존하고최종selected-headcapture/identity와구별한다. 이최종cleanHEAD의원M1전체41suite를새ignoredconfig/같은최소인자로다시실행하고단일native결과를봉인한다.

## Quality P06-3 first candidate checkpoint (implementation, finalquality/independentverification pending)

LatestDIRECTIVES/usercity/sea/20k–30k andvalid2d920P05-2F03/F04read,sourceab64clean. SameownedWT, no newapps/agents/mainmerge/push/publichosting/globalsettings. New evidence exclusivelytarget/evidence/WORLD-PREVIEW-quality-P06-3; priorab64all394/rootident andparentcustody/oldcandidatefailures unchanged. Ownscope maps/preview/ownasset/locales/docs only; game/proto/network/crates/packs/goldens/oldtests/checkers/locks/GLBs/recipes protected.

Firstaudit ab64-coverage.json native0:Irelandisland5789px/82,360.65km²sphereapprox/19sourcecitypoints/ID1,largestshare1.0;GB15/Hokkaido1/Shikoku1/Honshu+Kyushu17. Source4conn is geographicraster, not territories; modernsourcejoinsKyushu/Honshu,Eurasia/Africa,Americas, explicitlimitations. Original1pixel6500 versus actualkindcomponent1pixel6125(land2366/sea2267/lake1492). Urbanthresholdmeasurements/officialZIP4.1 vsweb4.0/CRC/date/PRJ/terms/SHA innewsource-manifest andreceipts.

REDcoverage-red6missingfunction/noanchorERROR→coverage-green-first5pass/1FAIL(theownnewtestdraft3componentassumptionwasincorrect)→independentnoanchor-geometryfloodfill/land2lake2/exactIDs/adja andstrengthenedcoverage-green-strengthened6PASS. Existingexpected/goldens untouched. Clienturbancontract-rednative1(invalidnegativeareaaccepted)→client-urban-contract-greennative0/7tests. Typecheck0. No primitivecitypoint-to-urbanbuffer; actualfootprints only.

Candidate1boundednative0/109.047s/peakWS1,638,838,272/peakCommit2,640,060,416bytes<180s/3GiB. 8192×4096/26,213IDs=land14,860/sea7,300/lake4,053;50,694adjacency;647actualrepresentativeDEM. allpixelkind/allrowurbanpixels/area/bbox/DEM/4conn/seamadja/source7SHA selfchecknative0(candidate-1-validation.json). Ireland21/largestshare.171705 includes8IDs<9px and1pixel2: not21equallyusefulparcels. GB71/Hokkaido16/Shikoku10/Honshu+Kyushu191. Onepixel3168land/2479sea/1492lake=7139,+639fromab64; sourcekind1pixel6125unchanged. Candidate1originaloutputs/dirtyfirstproof preserve andsmallgeneratedfragments needfurtherqualityreview.

Firstactual19PNG proof first-dirty-quality-proof native0/WebGPU NVIDIATuring8192,ownminimal4argv/native0/HTTP400+/pageerror/console0/unexplained0;rawStrictModeabortsseparate. rendererab64+dirtyrecipe sourceSHA/copies in source-receipt.json/source-snapshot,inputDircandidate1route. Dublinactualpointselected24347/24px/342.17km²/fullsourceurban1.0;London/Seoul/Tokyoactualpointparcels also1.0. IrishSeaactualclick6384/26,952.91km²,Channel4232/24,393.22,Atlantic6874/107,763.12 eachkindsea/uniqueID. CurrenteraMODIS2002–03/generalizedsource/otherfootprints unseparated UIclear; no rural/suburbanauthority. Parentreceivedfirstdirtyproof immediately, notfinalclean/P05PASS. Sourcekind&sameactualurbanclass generated-fragmentcleanupcandidate follows; no trueisland/waterhole deletion ordisconnectedmerge/target-counttrimming.

Decisionneeded:nonewithinscope. Unfinished:generatedfragmentquality/fullregressions/finalsame-source2bakes/nativeartifactZIPseal/independentP05/mainCI;globalDEM/precisechannels/ridgepass/urbanhistoric/current2026classification/gamefunctions remainunconnected. Boundedcandidate2repeat started sameactualgenerator beforefuturequality edits. Finalbakepairs identifiedbyexactrecipe, notmergedacrossattempts.

첫locale실행은cargoPATH미설정native1(FileNotFoundError)로원로그보존,기존C:/Users/user/.cargo/bin을해당프로세스PATH에만추가하여별도native결과확인한다. 전역설정/원checker변경없음. Candidate1/2 같은generator/options/tools 출력4SHA전부일치(candidate-1-2-repeat.json);107.297s반복native0/peakCommit2,639,507,456B. 원source가아닌fragmentquality추가후finalpair와구분한다.

### P06-3 내부 조각 정리·최종 생성 소스

동일 raw seed의 최대 연결 조각과 whole-seed를 보존한다. 나머지 ≤8pixel 조각 중 kind/실제 urban mask/eligible urban class가 균일하고 >8pixel 동일 class 4-neighbour 대상이 있는 조각만 공유 edge→대상 크기→ordinal 순으로 재할당한다. 원 지리 섬·물 구멍 삭제, 끊어진 ID 병합, dateline ID 병합, 숫자 맞추기 trimming 없음. 조각 RED missinghelper native1→GREEN8tests0, 통계 명명 RED missingkey native1→GREEN9tests0. ≤8pixel 이웃만 있는 fixture와 외부 같은 표면 이웃이 없는 fixture를 별도로 검사한다.

후보3/4 native0/160.156·156.750s 및 원 metadata/반복4SHA는 이전 통계 명명 상태로 보존한다. preserved_surface_islands=11840라는 초기 이름은 실제 섬을 세지 않아 부정확했다. 최종 소스는 preserved_without_large_same_surface_target=11840를 only-small-neighbours427/no-external-same-surface-neighbour11413으로 나눈다. mixed38은 별도다. 정책과 좌표 변경 없이 진단만 정정했으며 원 기록을 덮어쓰지 않았다.

최종 소스 후보5/6 native0/176.000·147.016s, peakCommit2,639,302,656·2,638,712,832B/WS1,638,617,088·1,638,264,832B. 180s/3GiB 제한 유지; 첫 실행 여유4초로 좁아 성능 보증이 아니다. 네 출력 SHA 모두 일치(candidate-5-6-repeat.json), 첫 세 기하 출력은 후보3과 정확히 같다. 선택 데이터24,889=land13,882/sea6,954/lake4,053, adjacency47,554, 대표DEM620. onepixel6610=land2769/sea2349/lake1492는 후보1보다529 감소했지만 ab64보다110 많다. 원 kind 1pixel6125는 보존된다. 재할당1278조각/3435픽셀, 구면 근사53,636.84km²; 전체 kind/eligible urban class 변경0/도시 observed pixels57,247 유지(candidate1-to3-repair-invariants.json). 최종 exhaustive candidate-5-validation native0/source7SHA/모든 픽셀 kind/4conn/면적/대표DEM/bbox/pixels/adjacency/seam 검사를 직접 실행했다.

최종 coverage: Ireland 섬17IDs/5789px/최대17.1705%, GB68, Honshu+Kyushu173(원8K 좁은 수로 누락으로 연결되어 같은 component), Hokkaido16, Shikoku10. 국가 영토나 전부 유용한 크기의 프로빈스로 해석하지 않는다. 도시 actualpoint Dublin23023/342.17km²/24px, London23080/2392.75/161, Seoul23751/2566.56/136, Tokyo23828/1550.20/80는 원 시가지 비율1.0. 1500km²는 시드당 명목 평균으로 최대 면적 제한이 아니다. 기타 원 작은 시가지/전체 DEM·정확 해협·능선·고개·현대2026/역사1936 분류·게임 기능 미완료를 유지한다.

CEO ProvGen README/Settings/review만 읽어 입력 역할을 docs/plans/WORLD-PREVIEW-ProvGen-comparison.md에 대조했다. ZIP 재조사/EXE 실행·디컴파일/포함 BMP·코드·팔레트·설정값 채택·새 importer/정식 포맷 없음. CRC는 DBF를 포함한 ZIP 모든 member의 raw bytes를 검사하지만 DBF 속성은 파싱/채택하지 않는다는 원천 설명을 명확히 했다.

최종 소스 자체검사 client140/Python27/build/typecheck/103dependency licenses/docs/assets/Fluent0, Rustfmt/clippy/workspace0, M0b039d35666b77fc2×2/M1b595dc2a1e5b4f8c×2. 첫 Rust helper는 Windows executable 검색 경로 오류로 시작 전 native1이며 Cargo 절대 경로 재실행 결과와 구별한다. 원 회귀/기대값/체커/timeout/retry 변경 없음. 기존 bundle-size/unused locale 경고 유지. 원M1 최소 Chrome 단일 전체·최종 실화면/선택/통신·clean identity/ZIP은 이 체크포인트 이후 직접 실행하여 같은 새 증거 폴더에 기록한다. 독립P05/mainCI/WP32/45/M5 판정은 선언하지 않는다.

### P06-3 clean ed0171f 실제 자체검수 결과

원M1 41tests 단일 전체 native0/41expected/0skip·flaky·unexpected/44.805s. 첫 client 외부 cwd 실행은 원 config의 resolve가 잘못된 서버 경로를 찾은 native1/서버 시작 전 실패로 m1-first-wrong-cwd에 보존; 올바른 지정 worktree/client cwd에서 원 config·fixtures·assertions·timeout·retry를 바꾸지 않고 재실행했다. 실제 Chrome argv는 headless/hide-scrollbars/ownprofile/pipe/no-startup-window만이다. Browser stderr의 Chrome updater named-pipe 접근 오류 원문은 로그에 보존하며 page error나 테스트 실패로 지우거나 혼동하지 않는다.

clean ed0171f의 checkpoint-fragments-full17PNG/quality19PNG native0: 실제 WebGPU NVIDIA Turing max8192와 WebGL2 ANGLE GTX1660Ti/D3D11 max16384, GLB6/912triangles, 전 카메라/hover·pick/zoom·pan/경계 전환/색 모드/ko·en/850×650 scrollrail/region-legend 비겹침/1280×720/1280×800 확인. 실제 inspector가 작은 viewport에서 스크롤 접근되고 map/화면은 검게 나오지 않았다. HTTP400+/console error/page exception/unexplained request failure0; rawStrictMode ERR_ABORTED는 full4건을 actual signal-aborted trace4건과 대조했다. Browser parent PeakWS165,994,496B는 GPU/renderer child 및 JSheap 합계가 아니다. 실제 도시4개와 해상3개 클릭값을 원 index bytes와 직접 대조(browser-validation.json); IrishSea6040/26,952.91km², Channel3987/24,393.22, Atlantic6528/107,763.12는 모두 kindsea/uniqueID다. 도시 원 시가지 비율1.0 유지.

tiny-selection 실제 1pixel 표본: Dublin 프리셋에서 이동한 source-raster isolatedland15287(-6.921387,52.141113)/14.65km²와 Tokyo 근처 samekind land 이웃이 있는15621(140.295410,35.354004)/19.47km²를 기존 zoom60에서 실제 hover/pick했다. 표시폭·높이 각각11.015625screenpixels, 선택값 정확. 실제 물리적 섬의 측량 검증이나 world/region zoom에서 쉬운 선택을 증명한 것은 아니다. 이웃 존재만으로 원 source fragment/repair eligibility를 확정하지 않는다. 첫 ignored helper import 경로 오류 native1은 tiny-first-import 원문/소스에 보존하고 올바른 경로 native0와 구분한다. no pixel 삭제/원 kind 섬6125 보존/생성 onepixel 남음6610이라는 한계를 유지한다.

CDP socket-probe native0: actual Vite HMR1(protocolvite-hmr), game/ws0, unclassified0; 전체 WebSocket이0이라는 주장은 하지 않는다. ed0171f는 clean actual source이며 이후 이 기록 문서 변경은 제품 SHA를 바꾸지 않는다. 최종 clean HEAD에서 quality·동일 최종 renderer의 ab64 baked 입력 비교 PNG를 새 final 폴더에 다시 기록하고 identity·SHA manifest·모든 member 검증 ZIP을 봉인한다. 원2d920/ab64 identity와 모든 원 실패·후보는 그대로 보존한다. 부모의 독립 검증/기본 브랜치 CI 판정은 별도다. 결정 필요/범위 밖 발견은 이 소유 작업 내 없음; 알려진 source/가독성/게임 미연결 한계는 위와 같다.

## Performance P06-4 — 원446·유효 P05-3 F05 보존

권위 docs/verify/WORLD-PREVIEW.M2-r2.attempt3.md 전문을 읽었다. 유효 FAIL은 원446 boundedfresh1=174.156s/native0/4bytes동일, fresh2=180.078s/native1, 추가fresh3=180.188s/native1 time budget exceeded이며 독립 반복 미충족이다. 다른 정확성·화면·3browser 각41이 통과한 것과 전체FAIL을 구분한다. 원 검증 worktree/index/fullraw/리포트/producer446 ZIP/과거 실패·봉인은 수정하지 않았다. 새 증거는 target/evidence/WORLD-PREVIEW-performance-P06-4만 쓴다. source446/동일branch·단일소유/새앱·위임 없음. source7/defines/데이터4/client/GLB/locale/감독·기존테스트·체커·crates/locks/CI·게임규칙 보호경로 그대로다.

대표 프로파일을 먼저 측정해 ADR-3202를 작성했다. connected_ids 원 count별 전체bbox 비교를 label→uint16 ordinal LUT/배경 조건 copy 한 번으로 바꾸고 repair firstcell argwhere의 전체좌표 할당을 row-major argmax/divmod로 바꿨다. 원 seed/component 순서·빈slot·배경·dtype/stride·1based65535·0based65534·overflow거부·fragment 선택/records 순서와 metadata를 보존한다. LUT 최댓값 검사 전 캐스팅/쓰기 없음. 기하·시드·소스·수치·메타 버전 변경 없음.

새 의미 oracle는 독립4neighbor floodfill이다. 원 RED 첫 draft의 자기 literal이 seed9 row-major 시작 순서를 잘못 적어 추가1FAIL이었다; draft/실패는 보존하고 명시된 좌표 목록과 floodfill·원446 일치로 정정했다. strengthened RED6tests 중 비용2FAIL/새firstcell helper1ERROR, 나머지 의미검사PASS→GREEN6PASS. 원fixture/테스트/기대값을 약하게 바꾸지 않았다. compare 작업량은 같은box1840pixel 입력에서 원99520pixel 비교를 거부하며 새 one-mask-per-seed가 통과했다. signed/unsigned/비연속·전치/offsetbox/멀리 떨어진 같은 seed/빈slot/65535허용·65536거부/첫셀 전체좌표 할당을 실제 검사한다. 원 F03 zero-edge/noanchor/F04/coverage/mixedsurface/tie/dateline 검사도 그대로 유지했다.

새 fresh1 PID29428/native0/72.438s/PeakCommit2,639,233,024/WS1,638,522,880B, fresh2 PID15328/native0/71.594s/PeakCommit2,640,277,504/WS1,639,227,392B. 원180s·3GiB·64MiB 그대로, 여유107.562·108.406s. 기존산출물을 복사해 fresh로 세지 않았다. 원CLI가7고정입력에서 새 ignored output에 전체 오프라인 생성을 두 번 실행했다. 비교 전용 historical446-baked만 git에서 읽어 만들고 이를 fresh로 표시하지 않는다. 원446·shipped·fresh1·fresh2 네 파일의 bytes/SHA 모두 동일, metadata/debug/source/defines/tools/epoch도 그대로(fresh-pair-contract.json). pair 중 이 세션은 build/test/cProfile/browser 병행 없이 작은 문서/hash 작업만, 다른앱 부하는 미측정·미제어. 독립 원검증의 일부회귀·화면 병행과 다른 조건이므로 모든환경 성능 보장/동일부하 benchmark로 해석하지 않는다.

새 fresh1 실제33554432pixel sourcekind·urbanpixels/면적·bounds·대표DEM620/24889모든ID4conn/adjacency47554/seam·source7SHA 검수 native0. coverage는 원17Ireland(13≥9/4tiny)/GB68/Honshu+Kyushu173원수로연결/Hokkaido16/Shikoku10/도시4fraction1.0/1pixel6610·sourcekind6125 그대로다. fragment1278record/3435pixel을 원candidate1과 직접 대조하고 원 onebased debug ID/동일 표면 계수edge/대상>8/최종ID를 검사했다. counter11840=427/11413,mixed38과 target ordinary의 다른 위치 <100km² 미분리urban 한계도 보존한다. geometry/record 변경을 주장하지 않는다.

자체회귀 Python33/native0(기존27+새6), Rustfmt/clippy/workspace와 M0b039d35666b77fc2×2/M1b595dc2a1e5b4f8c×2/native0; build/typecheck/103dependencylicense/docs/assets/Fluent0. pair 후 병행 Rust/Python/client/전수검사 중 원client palette test가5170ms/5000ms 제한으로1FAIL·139PASS/native1이었다. 원로그·native를 보존하고 원테스트/설정/timeout 변경 없이 별도 단일 전체 실행client140PASS/native0(client-isolated-full)를 직접 얻었다. 이 병행 부하의 범위 밖 관측은 부모에 전달하며 palette/client코드를 바꾸지 않는다. 초기 SHA/config-copy inline helper의 quoting parse 오류2건도 native1을 기록하고 실제 성공 receipt와 구별한다.

이 체크포인트 이후 원M1 최소 Chrome 단일전체41과 same-source fresh입력 world/Ireland/도시/sea/지역/3D/원446 동일카메라 비교·socket/원생산자ZIP identity/critical/artifact ZIP 봉인을 직접 남긴다. 클라이언트·data4를 새생성으로 덮지 않으며 exactbytes와 page inputDir로 직접 대조한다. 기존게임 미연결/globalDEM/원수로·Caspian/현대2002–03 일반화·tiny 한계 유지. 결정필요 없음; 범위밖관측은 병행clientpalette timeout 위1건(미수정). 독립 PASS/mainCI·WP/M5/완료를 선언하지 않는다.
