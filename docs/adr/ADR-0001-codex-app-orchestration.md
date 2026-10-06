# ADR-0001 Codex 앱 채팅과 전용 Git worktree 운영

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-06 |
| 관련 WP·REQ | WP-06, REQ-LEG-01 |

## 맥락

02 §12.4의 실제 CLI 환경 시험을 실행했다. 실행 도구는 Windows의 Git Bash(`C:/Program Files/Git/bin/bash.exe`)와 Codex CLI 0.144.1이었다. 하위 모델은 기존 설정의 gpt-6.1-sol이며, 인증은 기존 ChatGPT 로그인이다.

`tools/orch.sh selftest`의 최초 외부 PowerShell 호출은 도구에서 종료 코드 1로 관측됐다. 문서 검증 중 같은 설정의 Git Bash 호출을 재실행해 LASTEXITCODE를 직접 저장했으며 selftest 종료 코드 2를 확인했다. 하위 Codex 호출은 종료 코드 1이었다. 반복 증거는 docs/worklog/evidence/WP-06/selftest-repeat.* 에 보존했다. 서버 응답은 HTTP 400 `The 'gpt-6.1-sol' model is not supported when using Codex with a ChatGPT account.`였다. CLI의 일반 오류 안내와 달리 관측한 원인은 모델·인증 조합의 거절이며 네트워크 차단으로 단정하지 않는다. 로컬 MCP 포트 오류도 부수적으로 남았다.

`tools/orch.sh start impl WP-00 orch/selftest .orchestrator/prompts/WP-00.md`를 실행했다. 후속 별도 status 명령에서 running을 확인했고 최종 status는 done 1이었다. 분리 프로세스 생존은 확인했지만 정상 하위 실행과 done 0은 확인하지 못했다. 근거는 `.orchestrator/logs/selftest.err`, `WP-00.impl.{runner.log,err,jsonl}`, `.orchestrator/out/WP-00.impl.{state,exit}`다.

실패 시 실행 방식 질문에 사용자는 “현재 앱 설정으로 별도 채팅을 만들어 진행”을 선택했다. 사용자 지시는 이 작업의 실행 경로를 바꾸며, 미결정 게임·라이선스 결정을 바꾸지 않는다.

## 결정

OpenHOI M0에 한해 Codex 앱 create_thread/send_message_to_thread/wait_threads로 구현·검증 채팅을 조율한다. 각 WP는 별도의 Git worktree와 브랜치를 사용한다. 동시에 구현은 최대 두 채팅이며 독립 검증은 새 채팅에서 실행한다. CLI 모델·인증·승인·샌드박스 설정을 고치지 않는다.

앱 자동 worktree 요청은 clientThreadId `client-new-thread:c7dfdf29-3fd1-49fb-a43d-d3ac11104849`를 반환했다. `C:/Users/user/.codex/worktrees/788a/openhoi`의 detached checkout 생성은 관측했으나, 요청 직후 조회에서 실행 가능한 threadId가 나타나지 않았다. 이 초기 생성 요청은 정상 채팅 실행 증거로 간주하지 않는다.

대체로 Git으로 만든 전용 worktree를 앱 local 채팅 프롬프트에서 명시한다. 기본 cwd가 main일 수 있으므로 **모든 명령의 workdir와 Git -C는 해당 전용 worktree**로 지정하게 한다. 실제 첫 작업의 브랜치·경로를 확인한다.

앱 호출 시험 채팅 `01a11038-2afd-7dd3-9be4-753c94a02911`은 정상 종료했고 orch/selftest 브랜치, status 출력 없음, 각 명령 종료 코드 0과 OK를 반환했다. 오케스트레이터도 worktree의 clean status를 직접 확인한 뒤 사용자에게 이미 삭제가 승인된 WP-00 worktree와 orch/selftest 브랜치를 정리했다.

WP-01 구현 채팅은 `01a11038-c022-7860-a77f-8f0fe05c4b77`, 작업 경로는 `E:/openhoi/.orchestrator/wt/WP-01`, 브랜치는 `wp/01-skeleton`이다. 후속 채팅 ID와 실제 검증 상태는 작업 로그·검증 리포트·HANDOFF에 기록한다.

## 검토한 대안

| 대안 | 장점 | 버린 이유 |
|---|---|---|
| CLI 모델·인증 설정 수정 | 기존 orch.sh 운영 그대로 | 사용자가 앱 실행을 선택했고 임의 모델 변경은 하지 않음 |
| start 직후 wait | 종료 시 분리 프로세스 소멸 환경에서 사용 가능 | 관측한 실패는 모델 거절이며 wait로 해결되지 않음 |
| main을 구현 채팅끼리 공유 | 준비 단순 | 병렬 쓰기 충돌과 독립 검증 위반 |
| 오케스트레이터가 직접 구현 | 하위 실행 불필요 | 사용자 역할 지시와 D-15 위반 |

## 결과와 영향

검증 worktree는 구현 커밋을 detached checkout한다. 검증 시작 전 HEAD와 추적 파일 status를 기록하고 종료 후 HEAD 동일, 추적 파일 diff/status 없음, git ls-files 기반 목록 동일을 직접 확인한다. 파일 변경 또는 커밋이 있으면 무효다. 리포트는 채팅 마지막 메시지에서 오케스트레이터가 저장한다. 앱 실행에는 CLI `.exit`가 없으므로 앱 turn 완료/오류와 직접 실행 결과를 구분해 기록한다.

현재 오케스트레이터의 전달된 환경은 danger-full-access, approval policy never, 네트워크 활성이다. 이는 관측한 실행 문맥이며 설정을 변경한 결과가 아니다. 공식 설정 문서의 `sandbox_workspace_write.network_access`는 workspace-write 외부 네트워크 허용 boolean 키다. 이번 작업에서는 이 키를 설정하거나 기존 설정을 변경하지 않았다. 하위 앱 채팅의 실제 권한은 각 실행 문맥으로 확인하며 이 채팅과 같다고 추측하지 않는다.

출처: [공식 설정 참조](https://learn.chatgpt.com/docs/config-file/config-reference), [공식 Codex 앱의 별도 채팅·worktree 안내](https://developers.openai.com/blog/mastering-codex-remote-for-engineering). 확인일 2026-10-06. 게임 시뮬레이션 결정론·저장 포맷에는 영향이 없다.
