# WP14 a933 실제 Linux 소스 성능 증거

소스 `a9333419de89caf3ba95da5097ea2e3d0fa5f694`의 run 37695537881, attempt 1, job 113046236165에서 고정 prior 81deb와 현재 라이브러리를 별도로 release 빌드하여 동일 Linux runner에서 측정했다. baseline [195802551, 193460349] ns, current [192623408, 193688056] ns이며 엄격한 15% 초과 양쌍 회귀는 없다. 전체 DTO·canonical·hash `3b8853acbddce251`가 같고, 원 m1/seed1000/240000steps/warm2400/두 쌍/native240s/build1200s를 유지했다.

원 artifact 11514894569는 1735151692B, SHA256 `e83e3c48454aa3a282fa1361ea03fe20c8aa460d287ae29d80565bccf4b0b305`이며 원 ZIP의 754 파일을 전수 CRC·길이·SHA로 대조했다. prior 원 TAR 2,006,323,200B/SHA8ee7… 및 전체 입력 21파일·디렉터리 목록은 원 Git과 같다. 양쪽은 CEO의 명시적 최소 계측 예외를 적용한 같은 새 driver SHA9995…를 사용했고 실제 warmup과 네 표본의 pack_hash/pack_hash_after가 prior native hash3bde1bed90d3734e와 같다. 과거 드라이버 측정과 혼합하지 않았다.

원 metadata·path dependencies·registry version/checksum·job의 실제 build/측정 명령과 runtime identity guards를 대조했다. 변경하지 않은 workflow는 native 바이너리와 linked-source inventory를 업로드하지 않았으므로 환경의 binary SHA와 runtime 검사 증거를 물리 바이너리 원본 봉인이라고 주장하지 않는다. 이 작은 review ZIP은 API·환경·측정·전체 member inventory·104개 Range receipts와 수집 도구를 담으며 큰 원 ZIP/TAR는 명시한 ignored 원 경로에 보존한다. 다운로드 복구는 측정 재시도가 아니다.

부모 감사의 최초 CRLF checkout와 LF Git blob 직접 비교 실패(native1), 다음 외부 driver manifest 예외 누락 실패(native1)를 원 로그·도구로 보존했다. 새 namespace의 recovery2는 exact Git comparator와 원 bench/run.py 외부 workspace 경계로 실제 native0다. 제품·정책·워크플로·환경 설정을 바꾸지 않았다. 이는 소스 CI 증거이며 독립 P05 최종판정·P07·같은 기본 브랜치 CI 또는 신규 경제 활성 부하의 완료 증거를 대신하지 않는다.
