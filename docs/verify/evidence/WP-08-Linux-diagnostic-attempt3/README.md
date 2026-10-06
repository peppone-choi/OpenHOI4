# WP-08 Linux 진단 세 번째 한 회 결과

별도 f19 [37542615440](https://github.com/peppone-choi/OpenHOI4/actions/runs/37542615440)는 SUCCESS다. 고정sourceb73/strictc900, control M036→M182 / candidate M036→M182 각각 한 회, 재시도0·skip0이며 모든 구간 exit0·82사례 범위/순서 일치다. 제품 채택/새독립P05/main 최신CI/전체M1 완료는 아니다.

원 artifact11449133117은 8,380,918bytes/407members, API digest=ZIP SHA `1cd651a2c62e37b0bb8e9f16609934c3940b23ebc8672a7bb78f108a8c151404`다. original-wrapper.zip에 원 API·로그·artifactZIP·부모 대조를 보존했다. worker-analysis-original.zip은 읽기 진단 전문/분석/봉인 원문이며 REPORT.txt는 같은 원문 읽기본이다. 부모가 worker의460봉인파일 전부 길이/SHA를 직접 확인해 불일치0, 소스10스냅샷 전체 dictionary도 동일이다. raw index 자체 payload와 실행exe/dist payload·OS cwd는 Linux artifact에 없어 기록된 length/SHA와 실제 수집 코드를 대조한 범위다.

이번 고정순차pair의 Chromium preferred17.623→11.044초/forced15.451→10.756초, foreground SDK279→204/205, Frame.expect53→53이다. WebKit1.925→2.282초/1.718→1.813초, SDK84→73·expect18→18로 두 사례는 느려졌다. PNG28개 전체표본·rawGL·epoch·camera/authority·최종canvas/resources0을 대조했다. backend는 모두WebGL2로 이번 계측은 WebGPU 효과를 입증하지 않는다. 일부 live DOM/oldcontext 반환값은 따로 serialize되지 않아 원 source 단언과 PASS로 보완한 한계를 전문에 남겼다. 겹치는 SDK합계는 walltime이 아니며 CPU/driver 원인·일반화된 성능 개선·DPR 간헐문제 해결로 쓰지 않는다.

별도 같은f19 일반37542615505/Core37542615454/Sim37542615521는 SUCCESS이며 원fixture 정상CI다. main6b6 DPR 두30초FAIL와 SAV 검증 밖 Edge204/1 초기state 미해결은 그대로 보존한다. 이전 두 Linux 준비FAIL을 덮지 않았다. 추가push/run/retry/고립후속은 승인되지 않았다. 제품 적용은 이후 구체P06·새exact독립P05·동일mainCI로 검토한다.
