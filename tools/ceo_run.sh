#!/usr/bin/env bash
# OpenHOI CEO 도구 — 오케스트레이터 실행(codex exec)을 한 번에 하나씩 띄우고 관리한다. (docs/04)
#
#   tools/ceo_run.sh selftest                         CEO→오케스트레이터→하위 세션 중첩 호출 점검
#   tools/ceo_run.sh start <RUN> <프롬프트 파일>        오케스트레이터 실행을 분리 시작하고 바로 반환
#   tools/ceo_run.sh status                           실행 목록 + 구현·검증 세션 상태
#   tools/ceo_run.sh wait <RUN> [--timeout 초]          끝날 때까지 대기(기본 1800초). 끝나면 마지막 메시지 출력
#   tools/ceo_run.sh idle <RUN>                       마지막 로그 기록 후 지난 분(정체 감지용)
#   tools/ceo_run.sh stop <RUN>                       멈춘 실행 중지(오케스트레이터 프로세스 그룹만)
#   tools/ceo_run.sh run <RUN> <프롬프트 파일>          전경 실행(start가 내부에서 사용)
#
# RUN 형식: M<마일스톤>-r<회차>  예) M1-r1, M1-r2, M2-r1
# 오케스트레이터 샌드박스: 환경 변수 OH_ORCH_SANDBOX (read-only | workspace-write | danger-full-access)
#   기본값 workspace-write. 바꾸는 것은 사용자 결정 사항이다(AGENTS.md §3-3).
# 결과물 (.orchestrator/ 는 git에서 제외):
#   .orchestrator/ceo/<RUN>.{prompt.md,jsonl,err,runner.log,last.md,exit,state}
set -euo pipefail

die() { echo "ceo: $*" >&2; exit 2; }
ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || die "git 저장소 안에서 실행해야 함"
DIR="$ROOT/.orchestrator/ceo"
mkdir -p "$DIR"
EXCL="$(git -C "$ROOT" rev-parse --git-common-dir)/info/exclude"
mkdir -p "$(dirname "$EXCL")"
grep -qxF '.orchestrator/' "$EXCL" 2>/dev/null || echo '.orchestrator/' >> "$EXCL"

CODEX="${OH_CODEX:-codex}"
SANDBOX="${OH_ORCH_SANDBOX:-workspace-write}"
case "$SANDBOX" in read-only|workspace-write|danger-full-access) ;; *) die "OH_ORCH_SANDBOX 값 오류: $SANDBOX";; esac

check_run() { [[ "$1" =~ ^M[0-9]-r[0-9]+$ ]] || die "RUN 형식 오류: $1 (예: M1-r1)"; }

alive() {   # RUN → 0이면 실행 중
  local f="$DIR/$1.state" st pid
  [[ -f "$f" ]] || return 1
  read -r st pid < "$f" || return 1
  [[ "$st" == running && -n "${pid:-}" ]] && kill -0 "$pid" 2>/dev/null
}

running_runs() {
  shopt -s nullglob
  for f in "$DIR"/*.state; do
    r="$(basename "$f" .state)"; alive "$r" && echo "$r"
  done
  return 0
}

cmd="${1:-}"; shift || true
case "$cmd" in
  run)
    [[ $# -eq 2 ]] || die "사용: ceo_run.sh run <RUN> <프롬프트>"
    RUN="$1"; check_run "$RUN"; [[ -f "$2" ]] || die "프롬프트 파일 없음: $2"
    command -v "$CODEX" >/dev/null || die "codex CLI를 찾을 수 없음"
    cp "$2" "$DIR/$RUN.prompt.md"
    echo "running $$" > "$DIR/$RUN.state"
    rm -f "$DIR/$RUN.exit" "$DIR/$RUN.last.md"
    set +e
    ( cd "$ROOT" && "$CODEX" exec --sandbox "$SANDBOX" --json \
        -o "$DIR/$RUN.last.md" - < "$DIR/$RUN.prompt.md" \
        > "$DIR/$RUN.jsonl" 2> "$DIR/$RUN.err" )
    code=$?
    set -e
    echo "$code" > "$DIR/$RUN.exit"
    echo "done $code" > "$DIR/$RUN.state"
    echo "$RUN 종료: exit=$code"
    exit "$code"
    ;;
  start)
    [[ $# -eq 2 ]] || die "사용: ceo_run.sh start <RUN> <프롬프트>"
    RUN="$1"; check_run "$RUN"; [[ -f "$2" ]] || die "프롬프트 파일 없음: $2"
    [[ -e "$DIR/$RUN.state" ]] && die "$RUN은 이미 사용한 RUN 번호다. 회차를 올려라"
    others="$(running_runs)"
    [[ -z "$others" ]] || die "다른 오케스트레이터 실행이 진행 중: $others (한 번에 하나만)"
    echo "starting" > "$DIR/$RUN.state"
    if command -v setsid >/dev/null; then
      setsid nohup "$0" run "$RUN" "$2" > "$DIR/$RUN.runner.log" 2>&1 < /dev/null &
    else
      nohup "$0" run "$RUN" "$2" > "$DIR/$RUN.runner.log" 2>&1 < /dev/null &
    fi
    echo "$RUN 시작 (runner pid $!, sandbox=$SANDBOX)"
    ;;
  status)
    printf '%-10s %-8s %-5s %s\n' "RUN" "상태" "exit" "정체(분)"
    shopt -s nullglob
    for f in "$DIR"/*.state; do
      r="$(basename "$f" .state)"; read -r st pid < "$f" || true
      [[ "$st" == running ]] && ! alive "$r" && st="중단됨"
      ex="-"; [[ -f "$DIR/$r.exit" ]] && ex="$(cat "$DIR/$r.exit")"
      printf '%-10s %-8s %-5s %s\n' "$r" "$st" "$ex" "$("$0" idle "$r" 2>/dev/null || echo -)"
    done
    if [[ -x "$ROOT/tools/orch.sh" ]]; then echo; echo "[구현·검증 세션]"; "$ROOT/tools/orch.sh" status; fi
    ;;
  wait)
    [[ $# -ge 1 ]] || die "사용: ceo_run.sh wait <RUN> [--timeout 초]"
    RUN="$1"; check_run "$RUN"; shift; timeout=1800
    [[ "${1:-}" == --timeout ]] && timeout="${2:?}"
    start_s=$SECONDS
    while [[ ! -f "$DIR/$RUN.exit" ]]; do
      if ! alive "$RUN" && [[ "$(head -c 8 "$DIR/$RUN.state" 2>/dev/null)" != starting ]]; then
        echo "$RUN: 프로세스가 사라졌는데 종료 코드가 없음(중단됨)"; exit 6
      fi
      if (( SECONDS - start_s >= timeout )); then
        echo "$RUN: 아직 실행 중 (대기 ${timeout}s 초과, 정체 $("$0" idle "$RUN")분)"; exit 5
      fi
      sleep 10
    done
    echo "$RUN 종료: exit=$(cat "$DIR/$RUN.exit")"
    echo "----- 마지막 메시지 -----"
    [[ -s "$DIR/$RUN.last.md" ]] && cat "$DIR/$RUN.last.md" || echo "(없음)"
    ;;
  idle)
    [[ $# -eq 1 ]] || die "사용: ceo_run.sh idle <RUN>"
    RUN="$1"; check_run "$RUN"
    latest=0
    for f in "$DIR/$RUN.jsonl" "$DIR/$RUN.err"; do
      [[ -f "$f" ]] || continue
      m="$(stat -c %Y "$f" 2>/dev/null || stat -f %m "$f")"
      (( m > latest )) && latest=$m
    done
    (( latest == 0 )) && { echo "-"; exit 0; }
    echo $(( ( $(date +%s) - latest ) / 60 ))
    ;;
  stop)
    [[ $# -eq 1 ]] || die "사용: ceo_run.sh stop <RUN>"
    RUN="$1"; check_run "$RUN"
    alive "$RUN" || die "$RUN은 실행 중이 아님"
    read -r _ pid < "$DIR/$RUN.state"
    kill -TERM -- "-$pid" 2>/dev/null || kill -TERM "$pid"
    sleep 2
    kill -0 "$pid" 2>/dev/null && { kill -KILL -- "-$pid" 2>/dev/null || kill -KILL "$pid"; }
    echo "143" > "$DIR/$RUN.exit"; echo "done 143" > "$DIR/$RUN.state"
    echo "$RUN 중지함. 구현·검증 세션은 건드리지 않았다(orch.sh status로 확인)."
    ;;
  selftest)
    echo "1) git 저장소: $ROOT"
    command -v "$CODEX" >/dev/null || die "codex CLI 없음"
    echo "2) codex CLI: $(command -v "$CODEX")"
    echo "3) 오케스트레이터 샌드박스: $SANDBOX"
    [[ -x "$ROOT/tools/orch.sh" ]] && echo "4) tools/orch.sh: 있음" || die "tools/orch.sh 없음"
    p="$DIR/selftest.prompt.md"
    cat > "$p" <<'EOF'
역할: 점검. 다음 셸 명령을 그대로 실행하고, 그 표준 출력의 마지막 줄만 답으로 돌려줘. 다른 말은 쓰지 마.
codex exec --sandbox read-only "Reply with exactly: NESTED-OK"
EOF
    echo "5) 중첩 호출(오케스트레이터 → 하위 codex) 점검 중 …"
    out="$(cd "$ROOT" && "$CODEX" exec --sandbox "$SANDBOX" - < "$p" 2> "$DIR/selftest.err")" \
      || die "오케스트레이터 codex exec 실패 — $DIR/selftest.err 확인"
    if grep -q "NESTED-OK" <<< "$out"; then
      echo "   통과: 오케스트레이터 실행 안에서 하위 세션을 만들 수 있다"
    else
      echo "   실패: 하위 세션 응답이 없음 — 이 샌드박스로는 오케스트레이터가 구현 세션을 만들 수 없다"
      echo "   응답: $(tail -n 3 <<< "$out")"
      exit 7
    fi
    ;;
  *)
    sed -n '2,18p' "$0"; exit 2 ;;
esac
