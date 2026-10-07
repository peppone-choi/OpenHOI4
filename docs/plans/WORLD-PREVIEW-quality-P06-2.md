# WORLD-PREVIEW quality P06-2 조사·선택 계약

출발: clean2d920ea86c6d8045adb7d3115007230e52c3ccf2. 기존 체크포인트·실패·증거는 불변, 이번 증거는 target/evidence/WORLD-PREVIEW-quality-P06-2.

## 후보와 선택 순서

1. 한반도 지리 extent [122,32,132,43]에서512/1024/2048 raster, 실제NaturalEarth mask/river·고정 DEM·같은kind graph의 native 시간/PeakWorkingSet/PrivateBytes를 측정한다. 별도 프로세스 예산은180초/3GiB, 큰 후보는 측정 후 선택한다. 모든 값은 표시 제작 예산이며 게임 계수가 아니다.
2. 전세계 8192×4096 후보: RG8 index67,108,864bytes, JS 원bytes+decodedIndex최소134,217,728bytes, GPU index64MiB+256² lookup들이다. 장치actual maxTextureDimension2D가8192 이상일 때만 동작 가능. 가짜 GPU 제한 테스트와 실제 지원 GPU를 구별한다.
3. 전체 fine graph의 메모리 증가를 피하는 후보를 우선한다: 기존4096×2048 전역 samekind 지리 분할을 생성하고 실제8192 해안/호수 mask에 매핑한 뒤 한반도 등 작은지역만 fine graph+현대도시위치 density로 세분한다. 지역edge는 기존실제ID의boundary markers로연결해직사각지역cut을경계로삼지않는다. 지역새ID는전체registry에통합하고마지막4connected조각으로정규화한다. 전역ID는preview한정, 안정gameID/saveformat변경없음.
4. 측정 후 선택한 해상도/지역extent·density·minimumspacing/예산은 data/preview/world/defines.toml에 쓴다. 8k 상한 초과/IDs65535초과/누락입력/불일치/timeout/memory초과는 실패로 보고 원출력과기존파일을보존한다. 해상도 자동낮춤·다운로드fallback없음.

## 원천

기존land5.1.1/lakes5.0.0/rivers5.0.0/HimalayaDEM 고정원천 보존. [Natural Earth populated places simple5.1.2](https://www.naturalearthdata.com/downloads/10m-cultural-vectors/10m-populated-places/)의 공개 점 좌표만 추가 검토한다. [public-domain 조건](https://www.naturalearthdata.com/about/terms-of-use/). DBF·population·국가·도시게임기능은 사용하지 않는다. 목록이 지역 중요성을 기준으로 선별된 점이지 모든 도시/역사1936도시가 아님을 명시한다. 새 DEM은 필요 지역만 기존 official bounded F32 subset 방식으로 취득하고 좌표/수직기준/NoData를 보존한다. 입력이 없는 곳의 능선/고개를 발명하지 않는다.

## 기존 소비자와 검증

기존 RG8LE/dense index/256 lookup/OrthographicCamera/input/pipeline assertions/defaultApp 경로 보존. 새 preview dimensions 최대8192, u16 ID1..65535. renderer의optional presentation만 최소 수정 가능, oldfixture/expect/assert/timeout/retry/checker 수정 없음. source/metadata의 actual count/extent/hash·runtime SHA/ref 검증 및오프라인2회 재생성 유지.

한반도/Europe/하천·해안/산맥 동일 camera scale before2d920 baked inputs vs after를 같은최종renderer로비교하고 과거전체source검수와구별한다. 실제계단픽셀/반도횡단cell폭/실제도시주변cell수와source좌표대조를 기록한다. 3D샘플/HUD미연결/koen 유지. 기본GPU/security를보존한minimalChrome4옵션/ownPID/profile/127.0.0.1 pipe만 사용. 캡처는ownpage, 전역C UA/focus/input없음.

F01 locale원실패는2d920수정, 새검사실행. F02는기존전체suite 원기대값을동일scrollbar조건으로실행한다. scrollbar표시옵션은별도기록하며가속/보안switch를대체하지않는다. ERR_ABORTED는실제abort/cleanup/navigation 이벤트와 대조하고 HTTP/console 오류와합쳐0이라고쓰지않는다.

## 측정 후 선택 — 2026-10-07

지역실측512:0.406s/PeakWS97,808,384B;1024:1.25s/193,863,680B;2048:3.157s/580,513,792B. PrivateBytes1.06~1.12GB는설치된NumPy/SciPy기초process commit을포함하며WorkingSet과동일하다고쓰지않는다. 서로의실제phase/native0 및argv/region을원JSON에기록했다. 따라서worldfine8192×4096/coarsegraph4096×2048+smallfine지역후처리후보를선택한다. 전체8kgraph는채택하지않는다. 실제fullbake의3GiB/180s budget을별도로검사하고 초과면실패한다.

추가고정원천: NaturalEarthpopulatedplacessimple5.1.2 ZIP652145B/SHA2963ee08f7743f13f37d5243acd77a2f7a510e26a36c7e04b920f1b381f2b251,공식SOURCE embeddedVersion검사. NOAAKorea60arcsec F32 extent122,32,132,43/600×660 TIFF1396801B/SHAb5735d418e82f52b73c660052da77f189de85b67bc128169d9e666fa4ea12614. 30s/2MiB도시·30s/4MiB DEMbounded취득실제1.33/2.47s,원문로그보존. 아직지도적용전이다.

새fine지역후처리의markers는region바깥과이어지는기존kind일치edgeID+실제도시거리·하천거리·validDEM slope 가중독립seed이다. 지역extent는표시 window이며국경이아니다. density는Korea160추가landseed,Europe80/Himalaya60의작은지역대조후선택가능하고실제도시위치등입력이없는형상근거를발명하지않는다. 최종ID는새mask전체4connected재분리. 도시coords는성긴현대지리샘플이며역사/소유/인구값없음.

renderer에는preview전용 O(n)palette adapter와optional coast-kind-color interpolation을허용한다. 기존palettes/model/defaultApp은변경하지않는다. interpolation은표시filter만이며landmask/index/selection/adjacency는고정integer원본으로검사한다. kind가다른edge에만색interpolation을적용하고province내부경계는기존shader를사용한다. 해당option이없으면기존WP renderer 경로동일. 세계texture8192·denseRG8u16LE·256lookup유지. CPUloader불필요한64MiBcopy를줄이되DataViewLE해석과index검증보존.

### 측정 범위 정정 및 최종 후보

앞선0.406/1.25/3.157초 probe는메모리구조예비시험이며지역을전세계좌표로변환한metric/DEM미반영이었다. 실제DEM·지역extentmetric형상증거로전용하지않는다. 최종probe-*-final은지역cellcenter/실제KoreaDEM/정확extentmetric으로다시실행했다. evenly sampledseed메모리시험이며도시density형상검증과구별한다. native0실측: [{"size": 512, "seconds": 1.7659999999996217, "peak_working_set": 97636352, "private_bytes": 1065676800, "valid_dem_pixels": 262144}, {"size": 1024, "seconds": 2.186999999999898, "peak_working_set": 194600960, "private_bytes": 1085566976, "valid_dem_pixels": 1048576}, {"size": 2048, "seconds": 4.063000000000102, "peak_working_set": 579964928, "private_bytes": 1155244032, "valid_dem_pixels": 4194304}]。

실제fullcandidate5/6은84.766/86.203초,peakcommit2,637,049,856/2,637,975,552bytes,180초/3GiB이내. 최종같은입력/tools 4산출SHA전부일치. 전체8K소스kindpixel·20,366ID연결성·bbox·대표DEM·35,753sortedadjacency검사native0. 첫candidate1~4전환kind시험실패·초기mutablelabel출력은과거시도로만보존하고최종입력으로사용하지않는다. 원ZIP6/2subsetDEMsourceSHA는metadata와동일하다.
