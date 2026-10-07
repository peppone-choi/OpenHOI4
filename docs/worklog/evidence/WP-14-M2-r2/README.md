# WP-14 M2-r2 로컬 증거 봉인

제품 source: `01119ae05013a1974e5ca096c705bf39846c9cff`, 실제 Windows clean 실행. 이전 dirty/중간 source 및 실패는 별도 원 namespace다. 원 게임 규칙 확정·전체 REQ·독립 P05·Linux gate·actual3OS·main/dochead CI 판정은 포함하지 않는다.

`evidence.json`의 ZIP locator/SHA/길이와 BUNDLE_MANIFEST.json member 길이/SHA로 검증한다. ZIP은 원 출력·native receipts/저장·journal/replay·full DTO/canonical/hash/queue·입력/driver/source/binary provenance와 이전 실패를 포함한다. 모든 원문은 지정 WT의 ignored target/evidence에도 그대로 남았다.

벤치 prior 전체 Git archive와 원 archive stdout는 각2,006,323,200bytes이며 큰 원문을 ZIP에 중복하지 않았다. 해당 raw locator/SHA를 manifest에 명시한다. 전체 measurement testland 입력은 ZIP에 들어 있고 expanded source/build intermediate는 원 target에 남았다. 원/new driver bytes·부모 보류 diff·CEO 구체 인수 원문·인수 후 최종 diff를 구분한다. 역사 표본 소급 필드/혼합은 없다.

부모는 worktree 정리 전에 ignored 원 ZIP/큰 raw archive가 필요한지 확인하여 보존해야 한다. 제품 source 뒤 문서만 추가한 dochead 및 기본 브랜치 CI는 부모가 별도로 인수한다. 직접 원격 dispatch·main merge/push/다른 session 생성은 하지 않았다.
