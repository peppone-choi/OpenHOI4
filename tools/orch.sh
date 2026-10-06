#!/usr/bin/env bash
# OpenHOI 오케스트레이터 도구 — Codex 세션을 WP별 git worktree에서 만든다. (02 §12.4)
#
#   tools/orch.sh selftest                               실행 환경 점검(최초 1회)
#   tools/orch.sh start impl   <WP> <브랜치> <프롬프트>    구현 세션을 분리 실행하고 바로 반환
#   tools/orch.sh start verify <WP> <브랜치> <프롬프트>    검증 세션을 분리 실행하고 바로 반환
#   tools/orch.sh status                                 세션 상태 표
#   tools/orch.sh wait <WP>[:impl|:verify] ... [--timeout 초]   끝날 때까지 대기(대체 수단)
#   tools/orch.sh impl   <WP> <브랜치> <프롬프트>          전경 실행(start가 내부에서 사용)
#   tools/orch.sh verify <WP> <브랜치> <프롬프트>          전경 실행(start가 내부에서 사용)
#
# 결과물 (.orchestrator/ 는 git에서 제외된다)
#   prompts/  logs/<WP>.<mode>.{prompt.md,jsonl,err}
#   out/<WP>.<mode>.{last.md,exit,state,worktree}
#   wt/<WP>  wt/<WP>-verify    worktree
set -euo pipefail

die() { echo "orch: $*" >&2; exit 2; }
ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || die "git 저장소 안에서 실행해야 함"
ORCH="$ROOT/.orchestrator"
mkdir -p "$ORCH/prompts" "$ORCH/logs" "$ORCH/out" "$ORCH/wt"
# .gitignore가 아직 없어도 .orchestrator/가 커밋되지 않게 로컬 제외 목록에 넣는다
EXCL="$(git -C "$ROOT" rev-parse --git-common-dir)/info/exclude"
mkdir -p "$(dirname "$EXCL")"
grep -qxF '.orchestrator/' "$EXCL" 2>/dev/null || echo '.orchestrator/' >> "$EXCL"

check_wp() { [[ "$1" =~ ^WP-[0-9]{2}$ ]] || die "WP-ID 형식 오류: $1"; }
CODEX="${OH_CODEX:-codex}"

run_session() {   # mode WP branch prompt
  local mode="$1" wp="$2" branch="$3" prompt_in="$4"
  check_wp "$wp"
  [[ -f "$prompt_in" ]] || die "프롬프트 파일 없음: $prompt_in"
  command -v "$CODEX" >/dev/null || die "codex CLI를 찾을 수 없음"
  local tag="$wp.$mode" wt
  local prompt="$ORCH/logs/$tag.prompt.md"
  cp "$prompt_in" "$prompt"
  echo "running $$" > "$ORCH/out/$tag.state"
  rm -f "$ORCH/out/$tag.exit" "$ORCH/out/$tag.last.md"

  if [[ "$mode" == impl ]]; then
    wt="$ORCH/wt/$wp"
    if [[ -d "$wt" ]]; then
      echo "기존 worktree 재사용: $wt" >&2
    elif git -C "$ROOT" show-ref --verify --quiet "refs/heads/$branch"; then
      git -C "$ROOT" worktree add "$wt" "$branch" >&2
    else
      git -C "$ROOT" worktree add -b "$branch" "$wt" HEAD >&2
    fi
  else
    git -C "$ROOT" show-ref --verify --quiet "refs/heads/$branch" || die "검증할 브랜치 없음: $branch"
    wt="$ORCH/wt/$wp-verify"
    if [[ -d "$wt" ]]; then
      git -C "$wt" checkout --detach --force "$branch" >&2   # 구현 브랜치 최신 커밋으로
    else
      git -C "$ROOT" worktree add --detach "$wt" "$branch" >&2
    fi
  fi
  echo "$wt" > "$ORCH/out/$tag.worktree"

  # 샌드박스에서 네트워크가 막혀 있을 수 있으므로 의존성을 미리 받아 둔다
  if [[ -f "$wt/Cargo.toml" ]] && command -v cargo >/dev/null; then
    (cd "$wt" && { cargo fetch --locked 2>/dev/null || cargo fetch; }) >&2 \
      || echo "경고: cargo fetch 실패 — 오프라인 빌드가 실패할 수 있음" >&2
  fi

  local code
  set +e
  ( cd "$wt" && "$CODEX" exec --sandbox workspace-write --json \
      -o "$ORCH/out/$tag.last.md" - < "$prompt" \
      > "$ORCH/logs/$tag.jsonl" 2> "$ORCH/logs/$tag.err" )
  code=$?
  set -e

  # 검증 세션은 추적 파일을 바꾸면 안 된다 — 바꿨으면 검증 무효(종료 코드 4)
  if [[ "$mode" == verify && $code -eq 0 ]]; then
    if [[ -n "$(git -C "$wt" status --porcelain --untracked-files=no)" ]]; then
      echo "검증 무효: 검증 세션이 추적 파일을 수정함" >&2
      git -C "$wt" status --porcelain --untracked-files=no >&2
      code=4
    fi
  fi
  echo "$code" > "$ORCH/out/$tag.exit"
  echo "done $code" > "$ORCH/out/$tag.state"
  echo "$tag 종료: exit=$code worktree=$wt 마지막 메시지=$ORCH/out/$tag.last.md"
  return "$code"
}

cmd="${1:-}"; shift || true
case "$cmd" in
  impl|verify)
    [[ $# -eq 3 ]] || die "사용: orch.sh $cmd <WP> <브랜치> <프롬프트>"
    run_session "$cmd" "$@"
    ;;
  start)
    [[ $# -eq 4 ]] || die "사용: orch.sh start impl|verify <WP> <브랜치> <프롬프트>"
    mode="$1"; [[ "$mode" == impl || "$mode" == verify ]] || die "모드는 impl 또는 verify"
    check_wp "$2"
    [[ -f "$4" ]] || die "프롬프트 파일 없음: $4"
    tag="$2.$mode"
    if [[ -f "$ORCH/out/$tag.state" ]] && read -r st pid < "$ORCH/out/$tag.state" \
       && [[ "$st" == running ]] && kill -0 "$pid" 2>/dev/null; then
      die "$tag 이미 실행 중 (pid $pid)"
    fi
    echo "starting" > "$ORCH/out/$tag.state"
    if command -v setsid >/dev/null; then
      setsid nohup "$0" "$mode" "$2" "$3" "$4" > "$ORCH/logs/$tag.runner.log" 2>&1 < /dev/null &
    else
      nohup "$0" "$mode" "$2" "$3" "$4" > "$ORCH/logs/$tag.runner.log" 2>&1 < /dev/null &
    fi
    echo "$tag 시작 (runner pid $!). 'tools/orch.sh status'로 확인."
    ;;
  status)
    printf '%-18s %-9s %-5s %s\n' "세션" "상태" "exit" "마지막 메시지"
    shopt -s nullglob
    for f in "$ORCH"/out/*.state; do
      tag="$(basename "$f" .state)"; read -r st pid < "$f" || true
      if [[ "$st" == running ]] && ! kill -0 "$pid" 2>/dev/null; then st="중단됨"; fi
      ex="-"; [[ -f "$ORCH/out/$tag.exit" ]] && ex="$(cat "$ORCH/out/$tag.exit")"
      lm="-"; [[ -s "$ORCH/out/$tag.last.md" ]] && lm="out/$tag.last.md"
      printf '%-18s %-9s %-5s %s\n' "$tag" "$st" "$ex" "$lm"
    done
    ;;
  wait)
    timeout=0; tags=()
    while [[ $# -gt 0 ]]; do
      case "$1" in
        --timeout) timeout="$2"; shift 2 ;;
        *) t="$1"; [[ "$t" == *:* ]] && t="${t%%:*}.${t##*:}" || t="$t.impl"; tags+=("$t"); shift ;;
      esac
    done
    [[ ${#tags[@]} -gt 0 ]] || die "사용: orch.sh wait <WP>[:impl|:verify] ... [--timeout 초]"
    start_s=$SECONDS
    while :; do
      pending=0
      for t in "${tags[@]}"; do
        [[ -f "$ORCH/out/$t.exit" ]] || pending=1
      done
      [[ $pending -eq 0 ]] && break
      if [[ "$timeout" -gt 0 && $((SECONDS - start_s)) -ge "$timeout" ]]; then
        echo "시간 초과: 아직 끝나지 않은 세션이 있음"; "$0" status; exit 5
      fi
      sleep 5
    done
    "$0" status
    ;;
  selftest)
    echo "1) git 저장소: $ROOT"
    command -v "$CODEX" >/dev/null && echo "2) codex CLI: $(command -v "$CODEX")" || die "codex CLI 없음"
    command -v setsid >/dev/null && echo "3) setsid: 있음" || echo "3) setsid: 없음 (nohup만 사용)"
    printf '%s\n' "Reply with exactly: OK" > "$ORCH/prompts/WP-00.md"
    echo "4) 하위 Codex 세션 호출(네트워크·인증 확인) …"
    out="$(cd "$ROOT" && "$CODEX" exec --sandbox read-only - < "$ORCH/prompts/WP-00.md" 2>"$ORCH/logs/selftest.err")" \
      || die "하위 codex exec 실패 — 오케스트레이터 세션의 샌드박스가 네트워크를 막았거나 인증이 없음. logs/selftest.err 확인"
    echo "   응답: $(echo "$out" | tail -n1)"
    echo "5) 분리 실행 확인: 'tools/orch.sh start impl WP-00 orch/selftest .orchestrator/prompts/WP-00.md' 실행 후,"
    echo "   다음 명령에서 'tools/orch.sh status'로 WP-00.impl이 done 0이 되는지 확인한다."
    echo "   done이 안 되고 '중단됨'이면 분리 실행이 막힌 환경이다 → 02 §12.4의 대체 수단(wait)을 쓴다."
    ;;
  *)
    sed -n '2,16p' "$0"; exit 2 ;;
esac
