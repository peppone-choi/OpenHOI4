# WP-11 두 번째 독립 검증 실패 원본

exact750c의 새 앱/새 detached 검증 [전문](../../WP-11.attempt2.md)은 FAIL이다. 원래 M0 E2E의 기본 증거 경로가 추적 WP-12 자료40개를 다시 썼으며30개는내용변경/10개는동일bytes다. 최종 rawindex도달라졌다. 부모의 실제 Python postcheck exit4를 before/after JSON·원문index·명령/exit와함께 parent-before-after-bytes.zip에보존했다. 원파일/index를복원하거나before를교체하지않았다.

original-evidence-bytes.zip과preservation.json은target의원문기록/각명령 index/변경40파일 before·after/직접UI/native/기준계산/CI API를보존한다. 같은내용의원문바이트는ZIP에한번저장하고 sourcepath별 raw_archive_member로직접매핑한다. 부모는모든sourcepath의크기/SHA와대응member를대조했다. buildcache/의존성/별도source mirror는원래ignored위치에남아있고복제하지않았다. 제품검사성공과tracked+raw불변성FAIL를분리하며정식PASS·통합증거로사용하지않는다.

세번째검증은새same750 worktree/새앱에배정했으며모든producer의actualenv/cwd/default출력을먼저대조하고8증거env를M0/M1全browser에서ignoredtarget절대경로로전달하도록했다. 원시험/단언/시간/GPU/전체범위를변경하지않는다. 현재유효PASS나main통합을판정하지않는다.
