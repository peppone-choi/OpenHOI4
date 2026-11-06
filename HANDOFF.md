# 인수인계

한 쪽짜리 현재 상태다. 새 상태로 덮어쓴다. 2026-11-06 이전의 긴 기록은 git 이력(`e4e0c33` 이전 HANDOFF.md)에 있다.

| 항목 | 값 |
|---|---|
| 작성 시각 | 2026-11-06 |
| 현재 마일스톤 | M2 (Testland 수직 슬라이스), 남은 단위는 [BACKLOG](docs/plans/BACKLOG.md) §2 |
| 작업 브랜치 | `chore/m2-simplify-20261106` (main `6c20c5c` 기준, 미푸시) |
| 완료 판정 | CI 녹색 + PR 리뷰 1회 (D-15, 사용자 결정 2026-11-06) |
| 기본 브랜치 CI | 이 브랜치는 미확인 (아직 푸시하지 않음) |

## 이 브랜치에서 한 일

| 커밋 | 내용 |
|---|---|
| `e4e0c33` | 구현 단위 백로그 `docs/plans/BACKLOG.md` |
| `3dea11d` | 저장 형식 하나(v9)로 단일화, 예전 레이아웃·호환 fixture 제거 ([ADR-1001](docs/adr/ADR-1001-single-save-format.md)) |
| `b4e4699` | M2 팩 3개를 `testland_m2` 하나로 통합 (BACKLOG A1) |
| (이 커밋) | 절차 단순화: 독립 검증 세션·긴 HANDOFF 폐지, 완료 = CI 녹색 + PR 리뷰 1회 |

## 마지막 로컬 검증 (2026-11-06, macOS)

| 명령 | 결과 |
|---|---|
| `cargo test --workspace --no-fail-fast` | 전부 통과 |
| `oh_cli validate --deny-warnings data/packs/testland_m2` | 통과, hash `8b7603ecebb60e18` |
| `m2_military_query.cjs`, `o2_production_query.cjs`(fresh·restored) | 통과 |
| Playwright economy 8, m2-military 17, ready-deployment 8, training-controls 9 | 통과 |
| `python3 tools/check_docs.py`, `tools/check_assets.py` | 오류 0 |

## 알려진 문제

| 문제 | 재현 방법 |
|---|---|
| `military.spec.ts` 2개 실패(64·138행). 생성된 M1 fixture 호스트에서 패널 combobox가 3개라 strict mode 위반. 팩 통합과 무관한 호스트이며 main에서 재현 여부는 미확인 | `cargo run -p oh_save --example military_fixture -- target/wp22/military-ready` 후 `npx playwright test --config playwright.military.config.ts` |
| 로컬 Playwright가 chromium_headless_shell-1243을 찾지만 설치본은 1248 | `PLAYWRIGHT_CHROMIUM_EXECUTABLE`로 1248 지정 |
| military-ledger 3개, training-controls 1개는 예전 서버·번들 기준선 입력이 필요해 로컬에서 미실행 | `OH_LEGACY_SERVER_EXECUTABLE` 등 미설정 |

## 다음 세션이 처음 할 일

1. 사용자 확인 후 이 브랜치를 푸시하고 PR을 연다. CI 녹색과 리뷰 1회를 받는다.
2. BACKLOG C1(전쟁 상태 최소)부터 진행한다. 순서: C1 → B → C2 → D → E/F/G → H → I.
3. `military.spec.ts` 실패 2개가 main에서도 나는지 확인한다.
