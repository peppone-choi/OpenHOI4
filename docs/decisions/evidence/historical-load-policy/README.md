# 역사 데이터 로드 호환 후보의 원 입력

원 M0 pack 3파일과 원 immutable mutable-v1 fixture pack 19파일의 전체 상대경로/길이/SHA256 및 원 bytes를 보존했다. ZIP 22member는 actual source와 전수일치한다. FNV64는 기존 pack identity와의 대조 값이며 원 1byte 동일성의 단독 증명이 아니다. 후보 runtime 정책은 동일 파일목록·길이·SHA256를 전수 비교한 뒤에만 적용할 수 있다. 예외 채택/제품/독립 PASS 기록이 아니다. identity.json과 source-originals.zip이 provenance와 원본을 기록한다.

원 채택 전 REQUEST-0008 및 CEO 실제 독립 16조건 검토/부모 source cross-check 정본은 `review-originals.zip`이다. 원247724 요청과 현재 보강/채택 문서는 별개이며 source SHA/runtime 전수 조건으로 한정한다. 원정본 bytes는 ZIP으로 Git 줄바꿈 정규화와 분리한다.
