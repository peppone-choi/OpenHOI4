# WORLD-PREVIEW-M2-r2-P05-4 원 검증 증거

Git 단독 텍스트 사본은 저장소 줄바꿈 정규화가 적용될 수 있다. 원 API·prompt·최초/최종 JSON 및 raw index·manifest·ZIP identity의 정확 bytes는 parent-originals.zip에, 검증 payload 원 bytes는 evidence.zip orderedparts에 보존했다. 원본 ZIP은 검증자 경로에 유지했고 부모는 parts를 실제 재조립해 174,950,491B/SHA69c088b666c2e193e79ab235ea0d6becfcb4e22a9764149a81bce7ae87b7f6c2 및977개 member CRC를 확인했다. 실제 final15367chars 전문은 한정 PASS이며 원 F05 및 최초 wrapper 경로 관측을 면제·삭제하지 않는다.

exact `5794505abb2b0cfeaace8005183cb6606fc68301` 독립 새 앱의 실제 PASS 리포트 전문과 원 명령/산출물 ZIP을 보존한다. 부모는 최초 전체 추적 파일/SHA·HEAD·semantic/raw Git index·diff/cached/status와 976 manifest 파일 및 977 ZIP member의 원 bytes/SHA를 직접 대조했다. 불변성 유효 여부는 True이며 차이는 []다. False이면 이 검증은 무효이고 새 exact 독립 검증이 필요하다. 원 기준선·index를 복구하거나 대체하지 않았다. 새 source의 기본 브랜치 통합/같은 HEAD CI는 별도 P-07 증거다. 상세는 parent-custody.json과 원 전문에 있다.
