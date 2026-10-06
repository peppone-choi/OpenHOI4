# 지도 원천과 실행 경계 재확인

확인일 2026-10-06. M1 Testland는 자체 합성 지도이고 실제 세계/유럽 지도가 아니다. 이 기록은 원천 제한을 확대하지 않는다.

- [Natural Earth 공식 이용 조건](https://www.naturalearthdata.com/about/terms-of-use/): 사이트의 raster/vector 데이터는 public domain으로 안내한다. 실제 취득 파일의 고정 버전·해시·원천 기록은 WP-32/45에서 남긴다.
- [OpenHistoricalMap 프로젝트 라이선스 설명](https://wiki.openstreetmap.org/wiki/OpenHistoricalMap/License): 기본 CC0이며 별도 license 태그가 있는 요소가 존재한다. OSM에서 수입된 해안·수역·하천에 CC BY-SA 2.0 예외도 설명한다. raw OHM 전체를 CC0로 등록하지 않는다. 현재 원천 원칙에 맞는 CC0 검토 요소를 선별한다.

사용자 지시: 지리 원천 데이터를 웹에서 실시간으로 가져오지 말고 파일로 굽는다. 취득 단계에서 URL/일시/버전 또는 snapshot/라이선스/SHA256을 보존하고 오프라인 생성기만 그 파일을 읽는다. 런타임은 동봉 팩·자체 HTTP만 사용한다. 파일 누락·입력 해시 불일치는 실패로 노출하고 네트워크 다운로드로 대체하지 않는다. 원천·생성 옵션·도구 버전과 산출 해시를 기록해 같은 입력의 재생성을 대조한다. 실제 1936/1939 경계 자료 충족 여부는 후속 콘텐츠 조사에서 확인하며 현재 완성된 것으로 기록하지 않는다.
