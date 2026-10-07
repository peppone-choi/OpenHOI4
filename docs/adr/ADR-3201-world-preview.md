# ADR-3201 오프라인 세계 지도 시각 미리보기

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-07 |
| 관련 WP·REQ | WP-32, WP-45; REQ-MAP-01/02/04/07/10/11 일부 |

## 맥락

사용자가 실제 전세계 지도와 메인 HUD 외관을 기능 개발보다 먼저 보도록 직접 승인했다. 기존 Testland와 권위 서버를 변경하지 않고 독립 원천에서 생성한 시각 미리보기가 필요하다.

## 초기 체크포인트 결정

[최소 계약](../plans/WORLD-PREVIEW-schema.md)을 따른다. Natural Earth land/lakes ZIP를 SHA256 고정하고 오프라인 구면 nearest-seed 분할과 연결성 조각 분리로 굽는다. 기존 Three RG8 renderer/model을 display adapter에서 재사용한다. 4096×2048, 약 12,000 seed에서 시작하고 섬/호수 조각 수를 실제 기록한다. 주·국가·역사·게임 수치는 만들지 않는다. 첫 체크포인트에서는 고도 미적용을 명시한다.

## 검토한 대안

| 대안 | 장점 | 버린 이유 |
|---|---|---|
| 원작 지도 | 즉시 표시 | 클린룸 금지 |
| 직사각 격자 | 간단 | 사용자 비격자 요구 |
| 외부 실시간 타일 | 상세 지리 | 오프라인 원천 계약 위반 |
| DEM 전체 60 arc-second 파일 선행 | 전세계 높이 | 큰 다운로드가 첫 화면을 막음; 공식 subset을 후속 시도 |

## 결과와 영향

지리 제작 float는 오프라인 시각 데이터에 한정되며 sim DR을 변경하지 않는다. EPSG:4326 구면 거리 사용으로 높은 위도와 dateline 거리 왜곡을 줄이고, 표현은 Plate Carrée여서 극지 면적은 확대된다. raster 해상도 이하 섬/호수, 일반화 해안, near-coast aliasing, 게임 주 연결 미구현은 알려진 한계다. 저장/프로토콜/게임 모드 규격 변경 없음.

공식 확인(2026-10-07): [Natural Earth land v5.1.1](https://www.naturalearthdata.com/downloads/10m-physical-vectors/10m-land/), [lakes v5.0.0](https://www.naturalearthdata.com/downloads/10m-physical-vectors/10m-lakes/), [public domain 조건](https://www.naturalearthdata.com/about/terms-of-use/). NOAA [ETOPO2022](https://www.ncei.noaa.gov/products/etopo-global-relief-model)와 [Grid Extract](https://www.ncei.noaa.gov/maps/grid-extract/)는 subset/GeoTIFF를 제공하나 dateline 교차 및 ±89° 밖 요청이 제한된다. [메타데이터](https://www.ncei.noaa.gov/access/metadata/landing-page/bin/iso?id=gov.noaa.ngdc.mgg.dem%3Aetopo_2022)는 EPSG:4326/3855, CC0 조건을 안내한다.

설치된 numpy 2.2.6/scipy 1.17.1은 공식 version tag LICENSE의 BSD-3-Clause 확인 후 사용. Pillow 12.1.1은 MIT-CMU 표기이며 02 허용목록의 정확 SPDX와 다르므로 생성기 의존성으로 채택하지 않는다. SHP/PNG는 Python stdlib reader/writer로 구현하고 새 Python/Cargo/JS dependency를 설치하지 않는다.

## 후속 실제 지리 형상 입력

첫 e209/a28 체크포인트 이후 fixed Natural Earth rivers5.0.0/NOAA ETOPO2022 Himalaya TIFF를 도입했다. 현재 pipeline은 구면 nearest seeds를 초기 fallback으로 두고,8neighbor same-kind multi-source graph 최단거리로 actualriver/validDEM relief를 반영한다. seed없음인 작은 섬/component는 fallback 후4connected 조각으로 나눈다. 모든 allocation weights는 preview defines이며 sim 이동/전투에 전달하지 않는다. ridge 항은 validlocal elevation과 localminimum 차이이고 정확능선추출/고개검증 완료를 뜻하지 않는다. DEM밖/footprint경계를NoData상태로 구분하며 양끝점valid일때만높이cost를쓴다. terrain모드는대표cell표본 색이며 continuousrelief나globalDEM이 아니다. metadata와 UI가subset범위를명시한다.

source인수/strictTIFFparser/입력의형상효과/knowncoordinate/fullconnectivity/repeatSHA를직접검사했다. 공식 [SciPy dijkstra](https://docs.scipy.org/doc/scipy-1.17.0/reference/generated/scipy.sparse.csgraph.dijkstra.html)는 positive undirected weights/min_only/source mapping을지원한다. tie output은toolversion에영향받을수있어현재installed버전을고정·기록한다. [Natural Earth rivers](https://www.naturalearthdata.com/downloads/10m-physical-vectors/10m-rivers-lake-centerlines/) generalization/수로정렬·간헐하천누락과 sourceclass의Caspianlake미포함을한계로보존한다. 후속 detailedcity/DEM/riverpass검수 및게임주연결은미완료다.

공개플레이화면의정성적shape관찰은 SOURCES의격리browser증거로만남기고geographicgenerator입력에포함하지않았다. 원경계coordinate/픽셀분석/tracing나원게임/UI/아이콘/수치표사용없음.

## P06-2 technical extension (2026-10-07)

Choose bounded8Kcoast/coarse4Kglobalgraph/fine small-window refinement after512/1024/2048regional graph measurements. This avoids an all-world8Kgraph. Fixed modern point positions add geographic density only; no historical/game city fields. Korea adds fixed actual60arcsecDEM alongside earlierHimalaya75arcsecsubset. Complete globalDEM/ridge/pass precision remains outside the claim. Exact source bytes and acquisition terms are registered.

Retain integer IDtexture and default renderer contracts. A preview-only O(n)palette avoids repeated linear IDfind and an optional kind-color filter reduces coastal pixel visibility; picking still uses integer IDs. This is a display choice, not altered topology. Keep real proceduralGLBs/recipes unchanged. Add scoped rail scrolling and validate all camera presets. Bake process has180s/3GiBlimit and ownPIDreceipt; browser uses onlyheadless/ownprofile/pipe/no-startup-window. Original41M1Chrome tests use the original fixtures with documented hide-scrollbars to match original width expectations, preserving GPU/security defaults and all assertions/timeouts/retries.

Exhaustive source comparison found mutable fine-label reuse across kind passes. Reproduction preserved; corrected by reading labels from immutable coarse input. Candidate1/2/3/4 and corresponding draft captures remain attempt evidence; only later validated candidates are accepted. No automatic quality fallback, game rule, external runtime data, original boundary tracing or master base-prefix edits.

## P06-3 area coverage, urban masks and sea anchors

Follow latestuser20k–30k overall target(min20k;30knotabsolute;notland-only). Rawtotal alone cannot certifyquality. Source largecomponents had zeroseeds despitecitypoints, e.gIreland19points/oneID; constrained common area/density quota fixes this across all landmasses. Currentcandidate6000land/2500sea/1lake seeds are preview coefficients. Large≥10,000km² sourcecomponents receive minimum3seeds andmax65,000km² per allocatedseed floor; remainingweightedquota preservescoast/city/river/validDEM density. Rural12kmphysicalseedspacing/6×candidateoversampling limitsclose seeds. Sea1950coastalanchors with150kmspacing and550oceanseeds beyond250km useactualshoregeometry, sphere handlesdateline. Samekindgraph/actual8Kcoast/sourcekind/4connectedID/seamadja retained.

AddfixedNEurban4.1.0 fromMODIS2002–03; min100km² separate major footprints based on click-size/measurement1846rather than11967polygons. Largerfootprints average1500km² per source/coverage-guidedseed(max32) as an artgenerationcandidate, not gamecitycount. Allcoefficients are defines; no cityadministration/population/1936authority. Urbanmask/landkindintersection before separate rawIDs preserveswater and sourceholes. JSONarea_km2/urban_pixels/urban_partitioned are preview-only; sourceyear/omissions shown inlocale/HUD. Optionalsea-bordercolor changes onlypreviewpresentation; originalApp/shader/pipeline/backend/guards untouched. Advancedcamera/dropdown parameters are defines; same-rendereroldgeometry comparison addscurrentdisplaypresets only and recordsmetadatamutation separately.

Newnoanchorvalid1×4case hasfourcomponents land2/lake2; originaldrafttestexpected3 was wrong. Independentfloodfill andkindlabels prove4, exactdense[[0,2,1,3]]/seamadja strengthened; originalred6ERROR andgreen-first1FAIL remain. Sourcefinekindabsentfromcoarsemask now receivesindependentrawID andfinalconnectivity split. OriginalF03/F04 andoldtest/golden expectations remainunchanged.

### P06-3 fragment policy and research limits

Preserve largest connected piece of every raw seed and all whole-seed provinces. Transfer only non-largest ≤8pixel homogeneous kind/sourceurban/partitionedurban fragments to adjacent >8pixel stable components of identical class, ordering shared edges, target size, existing ordinal. No pixel/class deletion, disconnected/seam ID merge or count trimming. Candidate1 26,213→candidate3 24,889 reflects this policy, not a target-count cutoff. Preserved fragments with no large target are not all physical islands: 427 have only small same-surface neighbours; 11,413 have no external same-surface neighbour. Rename and split this diagnostic without changing coordinates/recipe; old candidate3/4 metadata stays immutable. Candidate5/6 are the final-source repeat pair.

Urban1500km² is nominal mean per allocated seed, not an enforced maximum parcel area. Candidate5 measured176s leaves only4s of the unchanged180s budget; this is a narrow observed margin, not performance assurance. ProvGen README/Settings compare input roles only; source/LICENSE and internal algorithms are unavailable. No ZIP reinspection/executable/BMP use, copied code/palette/settings, new gray importer or canonical mod/editor format. See WORLD-PREVIEW-ProvGen-comparison.md.
