# 지도 원천과 실행 경계 재확인

확인일 2026-10-06. M1 Testland는 자체 합성 지도이고 실제 세계/유럽 지도가 아니다. 이 기록은 원천 제한을 확대하지 않는다.

- [Natural Earth 공식 이용 조건](https://www.naturalearthdata.com/about/terms-of-use/): 사이트의 raster/vector 데이터는 public domain으로 안내한다. 실제 취득 파일의 고정 버전·해시·원천 기록은 WP-32/45에서 남긴다.
- [OpenHistoricalMap 프로젝트 라이선스 설명](https://wiki.openstreetmap.org/wiki/OpenHistoricalMap/License): 기본 CC0이며 별도 license 태그가 있는 요소가 존재한다. OSM에서 수입된 해안·수역·하천에 CC BY-SA 2.0 예외도 설명한다. raw OHM 전체를 CC0로 등록하지 않는다. 현재 원천 원칙에 맞는 CC0 검토 요소를 선별한다.

사용자 지시: 지리 원천 데이터를 웹에서 실시간으로 가져오지 말고 파일로 굽는다. 취득 단계에서 URL/일시/버전 또는 snapshot/라이선스/SHA256을 보존하고 오프라인 생성기만 그 파일을 읽는다. 런타임은 동봉 팩·자체 HTTP만 사용한다. 파일 누락·입력 해시 불일치는 실패로 노출하고 네트워크 다운로드로 대체하지 않는다. 원천·생성 옵션·도구 버전과 산출 해시를 기록해 같은 입력의 재생성을 대조한다. 실제 1936/1939 경계 자료 충족 여부는 후속 콘텐츠 조사에서 확인하며 현재 완성된 것으로 기록하지 않는다.

## 고도 후보와 사용자 참고 범위

[NOAA ETOPO2022 공식 제품](https://www.ncei.noaa.gov/products/etopo-global-relief-model)은 전세계 Ice Surface/Bedrock, 15/30/60 arc-second 해상도의 GeoTIFF/NetCDF와 부분 영역 취득을 안내한다. [공식 메타데이터](https://www.ncei.noaa.gov/access/metadata/landing-page/bin/iso?id=gov.noaa.ngdc.mgg.dem%3Aetopo_2022)의 Other Constraints에 CC0-1.0이 명시됐다. 후보 확인이며 아직 원천 파일을 취득하거나 기술 선택을 확정하지 않았다.

후속 ADR에서 해상도·용량·지역별 능선/고개 정확도·NoData·좌표/수직기준·단위·반복 산출 해시를 비교한다. 취득/생성 단계에서만 외부 자료에 접속하고 게임은 bake된 팩만 읽는다. 사용자 고도 요청의 범위로 필요한 독립 DEM을 추가할 수 있으며 다른 원작/지리 원천 제한을 확대하지 않는다.

사용자 확인 답 "공개 화면의 분할 원칙만 참고"는 공개 플레이 화면에서 산맥·강·도시 주변의 분할 원칙·밀도를 비교하는 예외다. 원작 경계 복사·트레이싱·지도 파일 추출/사용·수치표는 계속 금지한다. 실제 프로빈스 경계는 독립 지리·고도·하천 자료에서 생성하고 새 이동/전투 수치는 만들지 않는다.
# 후속 지형 검수 참고 — 2026-10-06

CEO의 추가 지시를 받고 [Natural Earth 하천 공식 페이지](https://www.naturalearthdata.com/downloads/10m-physical-vectors/10m-rivers-lake-centerlines/)를 직접 확인했다. v5.0.0 기본 하천·호수 중심선과 유럽 보충 자료가 후보이며 아직 취득하지 않았다. `10m`은 1:10 million 지도 축척이다. 해당 자료는 평활·위치 조정을 거쳤고, 일부 동북 러시아/아마존 정렬 의심과 간헐 하천 누락을 명시한다. 유럽 보충의 Natural Earth 가공 배포물은 public domain이나 그 설명에 연결된 원래 CCM의 더 상세한 선형은 비상업 이용 조건으로 구분된다. 원래 CCM을 같은 public-domain 허가로 자동 채택하지 않는다.

WP-32/45에서는 고정 원천 좌표계·그리드를 정합하고 고도·하천·프로빈스 경계·인접을 지역 확대 overlay로 독립 검수한다. 하천 연결과 호수/해안 진입, 산맥·능선·고개, 좁은 단절/인접 오류를 확인한다. DEM으로 계산한 배수로와 독립 하천 선형의 차이는 검토 대상이며 어느 쪽을 자동 정답으로 두지 않는다. 평균 고도만으로 검수를 대신하지 않는다.

M3 표본 후보는 알프스·피레네·카르파티아 및 라인·다뉴브·비스와 일대, M5 추가 표본은 해협·dateline·극지역이다. 이는 검수 준비 추천이며 알고리즘·실제 경계·새 지형 효과/계수의 승인이 아니다. 원천 버전·파일 용량·해상도·투영·NoData·수직 기준과 반복 산출 hash는 기존 취득/생성 ADR에 함께 기록한다.
