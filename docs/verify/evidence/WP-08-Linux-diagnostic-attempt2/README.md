# WP-08 두 번째 Linux 진단 준비 실패

정확6d1/sourceb73의 [진단37537984450](https://github.com/peppone-choi/OpenHOI4/actions/runs/37537984450)은 FAIL이다. 원producer130통과와M036통과 뒤, 이동한 ignored M1 config의 server cwd 기본값이 달라져상대 pack-root의manifest를못찾았다. M1 control은실제0case, candidate의두구간은미실행이다. DPR 비용/효과/PNG·rawGL 비교결과는없다. product 또는GPU결함으로분류하지않는다.

부모가직접내려받은 원artifact11446829246는3,127,577byte·100members·SHA256 d083c7bb86ca61d7d553ac8284e33bc7a6a7653d5e498f0a85bd513b4ce9414f로API digest와같다. original-evidence-bytes.zip은API·로그·원본artifact ZIPbytes를보존하고preservation.json이대조한다. 이번에는전체source-after와partial/error/미실행결과가있으며첫진단의전체after부재를덮지않는다.

[Playwright 공식 webServer 문서](https://playwright.dev/docs/test-webserver)의 cwd 기본값은config파일디렉터리다. 원본client config와복사target config가같은상대command를써도server cwd가달라진다. CLI의client cwd와server cwd를구분한다. source안pinned1.63 SDK와실제startup 로그도직접대조한다.

별도normalCI/Core/Sim의성공은임시진단결과와구분한다. 추가Linuxpush/run은승인하지않았고, 원server cwd를고정하는최소2파일ignored준비안에actual SDK loader/native startup/팩·exe·servedJS/ownPID·port정리와출력guard를먼저검사하도록배정했다. mock/list만으로준비성공을주장하지않는다. 과거2FAIL/미실행과GPU내부원인미확정은보존한다.
