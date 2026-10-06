#!/usr/bin/env python3
"""OpenHOI 문서 일관성 검사.

01 기획서 · 02 제작용 문서 · 03 제작용 프롬프트 · AGENTS.md · tools/orch.sh · 템플릿을
교차 검사한다. 오류가 있으면 종료 코드 1.

사용:
  python3 tools/check_docs.py                # 검사
  python3 tools/check_docs.py --write-trace  # 02 부록 A 추적 매트릭스 재생성 후 검사
"""
from __future__ import annotations

import re
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
F01 = ROOT / "docs/01-game-design.md"
F02 = ROOT / "docs/02-production-spec.md"
F03 = ROOT / "docs/03-production-prompts.md"
AGENTS = ROOT / "AGENTS.md"
ORCH = ROOT / "tools/orch.sh"
TEMPLATE_DIR = ROOT / "docs/templates"

MILESTONES = [f"M{i}" for i in range(7)]
VERIFY_CODES = {"UT", "IT", "GT", "DT", "SS", "PT", "BM", "DV", "CI", "RV"}
PRIORITIES = {"필수", "권장", "후순위"}
D_STATUS = {"잠정", "미결정", "확정", "폐기"}
ROLES = {"오케스트레이터", "구현", "콘텐츠"}

errors: list[str] = []
warnings: list[str] = []


def err(code: str, msg: str) -> None:
    errors.append(f"[{code}] {msg}")


def warn(code: str, msg: str) -> None:
    warnings.append(f"[{code}] {msg}")


def mnum(m: str) -> int:
    return int(m[1:])


def cells(line: str) -> list[str]:
    return [c.strip() for c in line.strip().strip("|").split("|")]


def split_ids(text: str, pattern: str) -> list[str]:
    return re.findall(pattern, text)


def read(p: Path) -> str:
    if not p.exists():
        err("FILE", f"파일 없음: {p.relative_to(ROOT)}")
        return ""
    return p.read_text(encoding="utf-8")


T01, T02, T03 = read(F01), read(F02), read(F03)
TAG = read(AGENTS)
TORCH = read(ORCH)
TEMPLATE_FILES = sorted(TEMPLATE_DIR.glob("*.md")) if TEMPLATE_DIR.exists() else []

ALL_DOCS: dict[str, str] = {
    "docs/01-game-design.md": T01,
    "docs/02-production-spec.md": T02,
    "docs/03-production-prompts.md": T03,
    "AGENTS.md": TAG,
}
for f in TEMPLATE_FILES:
    ALL_DOCS[str(f.relative_to(ROOT))] = f.read_text(encoding="utf-8")
README = ROOT / "README.md"
if README.exists():
    ALL_DOCS["README.md"] = README.read_text(encoding="utf-8")

# ---------------------------------------------------------------- 정의 수집


def collect_defs(text: str, id_re: str) -> dict[str, list[str]]:
    out: dict[str, list[str]] = {}
    for line in text.splitlines():
        m = re.match(rf"^\|\s*({id_re})\s*\|", line)
        if m:
            key = m.group(1)
            if key in out:
                err("C01", f"식별자 중복 정의: {key}")
            out[key] = cells(line)
    return out


D = collect_defs(T01, r"D-\d\d")
REQ = collect_defs(T01, r"REQ-[A-Z]+-\d\d")
OPEN = collect_defs(T01, r"OPEN-\d\d")
AC = collect_defs(T01, r"AC-M\d-\d\d")
WP = collect_defs(T02, r"WP-\d\d")
DR = collect_defs(T02, r"DR-\d\d")
P = {m.group(1): m.group(2) for m in re.finditer(r"^## (P-\d\d) (.+)$", T03, re.M)}

# ---------------------------------------------------------------- C09 D
for k, c in D.items():
    if len(c) != 5:
        err("C09", f"{k}: 열 수 {len(c)} (기대 5)")
        continue
    if c[3] not in D_STATUS:
        err("C09", f"{k}: 상태 '{c[3]}' 허용 안 됨")
    if c[3] == "미결정" and c[4] in ("", "—"):
        err("C09", f"{k}: 미결정인데 확정 필요 시점이 없음")

# ---------------------------------------------------------------- C10 REQ
req_ms: dict[str, str] = {}
for k, c in REQ.items():
    if len(c) != 5:
        err("C10", f"{k}: 열 수 {len(c)} (기대 5)")
        continue
    _, text, prio, ms, codes = c
    if prio not in PRIORITIES:
        err("C10", f"{k}: 우선순위 '{prio}'")
    if ms not in MILESTONES:
        err("C10", f"{k}: 마일스톤 '{ms}'")
    req_ms[k] = ms
    cs = [x.strip() for x in codes.split(",")]
    bad = [x for x in cs if x not in VERIFY_CODES]
    if bad or not cs:
        err("C10", f"{k}: 검증 코드 오류 {bad or '(없음)'}")

# ---------------------------------------------------------------- C08 OPEN
for k, c in OPEN.items():
    if len(c) != 4:
        err("C08", f"{k}: 열 수 {len(c)} (기대 4)")
        continue
    _, q, default, ms = c
    if ms not in MILESTONES:
        err("C08", f"{k}: 차단 마일스톤 '{ms}'")
        continue
    if mnum(ms) <= 3 and default in ("", "—"):
        err("C08", f"{k}: M3 이전을 차단하는데 기본안이 없음")

# ---------------------------------------------------------------- WP 파싱
wp_ms: dict[str, str] = {}
wp_wave: dict[str, tuple[int, int]] = {}
wp_deps: dict[str, list[str]] = {}
wp_reqs: dict[str, list[str]] = {}
wp_role: dict[str, str] = {}
for k, c in WP.items():
    if len(c) != 8:
        err("C05", f"{k}: 열 수 {len(c)} (기대 8)")
        continue
    _, name, ms, deps, wave, role, reqs, evid = c
    if ms not in MILESTONES:
        err("C05", f"{k}: 마일스톤 '{ms}'")
        continue
    wp_ms[k] = ms
    wm = re.fullmatch(r"(M\d)-W(\d+)", wave)
    if not wm:
        err("C05", f"{k}: 병렬 묶음 형식 오류 '{wave}'")
    else:
        if wm.group(1) != ms:
            err("C05", f"{k}: 병렬 묶음 {wave}의 마일스톤이 {ms}와 다름")
        wp_wave[k] = (mnum(ms), int(wm.group(2)))
    wp_deps[k] = split_ids(deps, r"WP-\d\d")
    wp_reqs[k] = split_ids(reqs, r"REQ-[A-Z]+-\d\d")
    if role not in ROLES:
        err("C18", f"{k}: 역할 '{role}' 허용 안 됨")
    wp_role[k] = role
    if not evid or evid == "—":
        err("C05", f"{k}: 완료 증거가 비어 있음")

# ---------------------------------------------------------------- C05 의존
for k, deps in wp_deps.items():
    for d in deps:
        if d not in wp_ms:
            err("C05", f"{k}: 선행 {d} 정의 없음")
            continue
        if mnum(wp_ms[d]) > mnum(wp_ms[k]):
            err("C05", f"{k}({wp_ms[k]}): 선행 {d}가 더 늦은 마일스톤 {wp_ms[d]}")
        if k in wp_wave and d in wp_wave and wp_wave[d] >= wp_wave[k]:
            err("C05", f"{k}: 선행 {d}의 병렬 묶음이 같거나 늦음 ({wp_wave[d]} >= {wp_wave[k]})")

# 순환 검사
state: dict[str, int] = {}


def dfs(n: str, stack: list[str]) -> None:
    state[n] = 1
    for d in wp_deps.get(n, []):
        if d not in wp_deps:
            continue
        if state.get(d) == 1:
            err("C05", f"의존 순환: {' → '.join(stack + [n, d])}")
        elif state.get(d) is None:
            dfs(d, stack + [n])
    state[n] = 2


for n in wp_deps:
    if state.get(n) is None:
        dfs(n, [])

# ---------------------------------------------------------------- C03 REQ 커버리지
covered: dict[str, list[str]] = defaultdict(list)
for k, reqs in wp_reqs.items():
    for r in reqs:
        if r not in REQ:
            err("C04", f"{k}: 대상 {r} 정의 없음")
            continue
        covered[r].append(k)
for r, ms in req_ms.items():
    wps = covered.get(r, [])
    if not wps:
        err("C03", f"{r}: 담당 WP 없음")
        continue
    if not any(mnum(wp_ms[w]) <= mnum(ms) for w in wps if w in wp_ms):
        err("C03", f"{r}({ms}): 같은 마일스톤 이전에 끝나는 WP가 없음 {wps}")

# ---------------------------------------------------------------- C06 마일스톤
for m in MILESTONES:
    if not any(k.startswith(f"AC-{m}-") for k in AC):
        err("C06", f"{m}: 완료 기준(AC) 없음")
    if not any(v == m for v in wp_ms.values()):
        err("C06", f"{m}: 작업 패키지 없음")

# ---------------------------------------------------------------- C07 AC
ac_reqs_all: set[str] = set()
all_ac_ms: set[str] = set()
for k, c in AC.items():
    if len(c) != 4:
        err("C07", f"{k}: 열 수 {len(c)} (기대 4)")
        continue
    am = re.match(r"AC-(M\d)-", k).group(1)
    rs = split_ids(c[3], r"REQ-[A-Z]+-\d\d")
    if "필수 REQ 전체" in c[3]:
        ms_in_cell = re.search(r"(M\d) 필수 REQ 전체", c[3])
        if not ms_in_cell or ms_in_cell.group(1) != am:
            err("C07", f"{k}: '필수 REQ 전체'의 마일스톤이 {am}와 다름")
        all_ac_ms.add(am)
        ac_reqs_all.update(r for r, cc in REQ.items() if cc[2] == "필수" and cc[3] == am)
        continue
    if not rs:
        err("C07", f"{k}: 관련 REQ 없음")
    for r in rs:
        ac_reqs_all.add(r)
        if r not in REQ:
            err("C07", f"{k}: {r} 정의 없음")
        elif mnum(req_ms[r]) > mnum(am):
            err("C07", f"{k}: {r}의 마일스톤 {req_ms[r]}이 {am}보다 늦음")
    if not c[2] or c[2] == "—":
        err("C07", f"{k}: 증거 칸이 비어 있음")
for m in MILESTONES:
    if m not in all_ac_ms:
        err("C07", f"{m}: '필수 REQ 전체' 완료 기준이 없음")
untested = [r for r, c in REQ.items() if c[2] == "필수" and r not in ac_reqs_all]
if untested:
    err("C07", f"어떤 완료 기준에도 걸리지 않은 필수 REQ: {', '.join(sorted(untested))}")

# ---------------------------------------------------------------- C02 참조 무결성
ID_PATTERNS = {
    "REQ": (r"REQ-[A-Z]+-\d\d", REQ),
    "WP": (r"WP-\d\d", WP),
    "D": (r"(?<![A-Za-z0-9-])D-\d\d(?!\d)", D),
    "OPEN": (r"OPEN-\d\d", OPEN),
    "AC": (r"AC-M\d-\d\d", AC),
    "DR": (r"DR-\d\d", DR),
    "P": (r"(?<![A-Za-z0-9-])P-\d\d(?!\d)", P),
}
for fname, text in ALL_DOCS.items():
    for kind, (pat, defs) in ID_PATTERNS.items():
        for mm in re.finditer(pat, text):
            if mm.group(0) not in defs and mm.group(0) != "WP-00":  # WP-00: selftest 예약
                line = text.count("\n", 0, mm.start()) + 1
                err("C02", f"{fname}:{line}: 정의되지 않은 {kind} 식별자 {mm.group(0)}")

# 범위 표기 "DR-01~DR-10", "P-01~P-12"가 정의 범위와 맞는지
for fname, text in ALL_DOCS.items():
    for mm in re.finditer(r"(DR|P)-(\d\d)~(?:\1-)?(\d\d)", text):
        kind, a, b = mm.group(1), int(mm.group(2)), int(mm.group(3))
        defs = DR if kind == "DR" else P
        for i in range(a, b + 1):
            if f"{kind}-{i:02d}" not in defs:
                err("C02", f"{fname}: 범위 {mm.group(0)} 안의 {kind}-{i:02d} 정의 없음")

# ---------------------------------------------------------------- C11 용어
gl_sec = re.search(r"^## 12\. 용어집(.*?)(?=^## |\Z)", T01, re.M | re.S)
forbidden: dict[str, str] = {}
if not gl_sec:
    err("C11", "01 §12 용어집을 찾지 못함")
else:
    for line in gl_sec.group(1).splitlines():
        if line.startswith("|") and not line.startswith("|---") and "사용 금지 표현" not in line:
            c = cells(line)
            if len(c) != 4:
                err("C11", f"용어집 열 수 오류: {line[:40]}")
                continue
            for f in [x.strip() for x in c[3].split(",")]:
                if f and f != "—":
                    forbidden[f] = c[0]
diff_sec = re.search(r"^## 11\. 원작 대비 설계 차별화 기록(.*?)(?=^## )", T01, re.M | re.S)
exempt_lines = set()
for sec in (gl_sec, diff_sec):
    if sec:
        exempt_lines.update(sec.group(1).splitlines())
for fname, text in ALL_DOCS.items():
    for i, line in enumerate(text.splitlines(), 1):
        if "원작" in line:
            continue
        if fname == "docs/01-game-design.md" and line in exempt_lines:
            continue
        for f, term in forbidden.items():
            if f in line:
                err("C11", f"{fname}:{i}: 사용 금지 표현 '{f}' → '{term}' 사용")

# ---------------------------------------------------------------- C12 프롬프트
var_sec = re.search(r"^### 0\.3 변수(.*?)(?=^### |^## |\Z)", T03, re.M | re.S)
declared = set(re.findall(r"`\{\{([A-Z_]+)\}\}`", var_sec.group(1))) if var_sec else set()
if not declared:
    err("C12", "03 §0.3 변수 표를 찾지 못함")
used = set(re.findall(r"\{\{([A-Z_]+)\}\}", T03.replace(var_sec.group(1) if var_sec else "", "")))
for v in sorted(used - declared):
    err("C12", f"선언되지 않은 변수 {{{{{v}}}}}")
for v in sorted(declared - used):
    warn("C12", f"선언했지만 쓰지 않는 변수 {{{{{v}}}}}")
sections = re.split(r"^## (?=P-\d\d )", T03, flags=re.M)
for s in sections[1:]:
    pid = s[:4]
    for field in ("사용 시점", "실행 주체", "입력 변수", "완료 조건"):
        if f"| {field} |" not in s:
            err("C12", f"{pid}: '{field}' 항목 없음")
    if "```text" not in s:
        err("C12", f"{pid}: 프롬프트 본문(```text) 없음")
    # 입력 변수 칸의 변수가 본문에서 쓰이는지
    m = re.search(r"\| 입력 변수 \|(.*)\|", s)
    if m:
        listed = set(re.findall(r"\{\{([A-Z_]+)\}\}", m.group(1)))
        body = s.split("```text", 1)[1] if "```text" in s else ""
        body_vars = set(re.findall(r"\{\{([A-Z_]+)\}\}", body))
        for v in sorted(body_vars - listed):
            err("C12", f"{pid}: 본문 변수 {{{{{v}}}}}가 입력 변수 칸에 없음")
        for v in sorted(listed - body_vars):
            warn("C12", f"{pid}: 입력 변수 {{{{{v}}}}}를 본문에서 쓰지 않음")
# 콘텐츠 역할 WP와 P-11 목록 일치 (C18)
p11 = re.search(r"^## P-11.*?(?=^## )", T03, re.M | re.S)
if p11:
    first_row = re.search(r"\| 사용 시점 \|(.*)\|", p11.group(0))
    listed = set(re.findall(r"WP-\d\d", first_row.group(1))) if first_row else set()
    content_wps = {k for k, r in wp_role.items() if r == "콘텐츠"}
    if listed != content_wps:
        err("C18", f"P-11 사용 시점 WP {sorted(listed)} ≠ 콘텐츠 역할 WP {sorted(content_wps)}")

# ---------------------------------------------------------------- C13 Codex 지침·오케스트레이터 도구
if TAG:
    n = len(TAG.splitlines())
    size = len(TAG.encode("utf-8"))
    if n > 200:
        err("C13", f"AGENTS.md {n}줄 (권장 200줄 이하)")
    if size > 32 * 1024:
        err("C13", f"AGENTS.md {size}바이트 — Codex 기본 로드 한도 32 KiB 초과")
    for role in ("오케스트레이터", "구현 세션", "검증 세션"):
        if role not in TAG:
            err("C13", f"AGENTS.md에 역할 '{role}' 규칙 없음")
subcmds = set()
if TORCH:
    for mm in re.finditer(r"^\s{2}([a-z|]+)\)", TORCH, re.M):
        subcmds.update(mm.group(1).split("|"))
for fname, text in ALL_DOCS.items():
    for mm in re.finditer(r"(?:tools/)?orch\.sh\s+([a-z]+)", text):
        if mm.group(1) not in subcmds:
            err("C13", f"{fname}: orch.sh에 없는 하위 명령 '{mm.group(1)}'")
    for mm in re.finditer(r"orch\.sh start\s+([a-z]+)", text):
        if mm.group(1) not in ("impl", "verify"):
            err("C13", f"{fname}: orch.sh start 모드 '{mm.group(1)}'는 impl 또는 verify여야 함")
LEFTOVER = ["oh-verifier", "oh-implementer", "oh-content", "CLAUDE.md", ".claude/agents", "Claude Code",
            "oh_godot", "oh_net", "dispatch_codex"]
for fname, text in ALL_DOCS.items():
    for i, line in enumerate(text.splitlines(), 1):
        for w in LEFTOVER:
            if w in line:
                err("C13", f"{fname}:{i}: 폐기된 구성 '{w}'가 남아 있음")

# ---------------------------------------------------------------- C14 템플릿·링크
for fname, text in ALL_DOCS.items():
    for mm in re.finditer(r"docs/templates/([A-Z_]+\.md)", text):
        if not (TEMPLATE_DIR / mm.group(1)).exists():
            err("C14", f"{fname}: 템플릿 없음 {mm.group(1)}")
    base = (ROOT / fname).parent
    for mm in re.finditer(r"\]\(([^)#\s]+\.md)\)", text):
        target = (base / mm.group(1)).resolve()
        if not target.exists():
            err("C14", f"{fname}: 링크 대상 없음 {mm.group(1)}")

# ---------------------------------------------------------------- C15 마크다운
for fname, text in ALL_DOCS.items():
    fences = len(re.findall(r"^```", text, re.M))
    if fences % 2:
        err("C15", f"{fname}: 코드 펜스 짝이 맞지 않음({fences})")
    in_code = False
    block: list[tuple[int, int]] = []

    def flush() -> None:
        if len(block) >= 2:
            want = block[0][1]
            for ln, n in block[1:]:
                if n != want:
                    err("C15", f"{fname}:{ln}: 표 열 수 {n} ≠ 머리글 {want}")
        block.clear()

    for i, line in enumerate(text.splitlines(), 1):
        if line.startswith("```"):
            in_code = not in_code
            flush()
            continue
        if in_code:
            continue
        if line.startswith("|"):
            # 셀 안의 \| 와 `...|...` 는 고려하지 않는다(문서에서 쓰지 않음)
            block.append((i, len(cells(line))))
        else:
            flush()
    flush()

# ---------------------------------------------------------------- C16 버전 일관성
VERSION_KEYS = {
    "Rust": r"Rust(?: \(stable\))?\s*\|?\s*(\d+\.\d+\.\d+)",
    "TypeScript": r"TypeScript\s*\|\s*(\d+\.\d+\.\d+)",
    "three": r"three\s*\|\s*(\d+\.\d+\.\d+)",
}
seen_versions: dict[str, set[str]] = defaultdict(set)
for fname, text in ALL_DOCS.items():
    for key, pat in VERSION_KEYS.items():
        for mm in re.finditer(pat, text):
            v = next(g for g in mm.groups() if g)
            seen_versions[key].add(v)
for key, vals in seen_versions.items():
    if len(vals) > 1:
        err("C16", f"{key} 버전 불일치: {sorted(vals)}")

# ---------------------------------------------------------------- C17 추적 매트릭스


def build_trace() -> str:
    rows = ["| REQ | 마일스톤 | 우선순위 | 구현 WP | 검증 |", "|---|---|---|---|---|"]

    def key(r: str):
        area = r.split("-")[1]
        return (list(dict.fromkeys(x.split("-")[1] for x in REQ)).index(area), r)

    for r in sorted(REQ, key=key):
        c = REQ[r]
        wps = ", ".join(sorted(covered.get(r, []))) or "**없음**"
        rows.append(f"| {r} | {c[3]} | {c[2]} | {wps} | {c[4]} |")
    return "\n".join(rows)


trace = build_trace()
tm = re.search(r"<!-- TRACE:BEGIN -->\n(.*?)<!-- TRACE:END -->", T02, re.S)
if "--write-trace" in sys.argv:
    new = re.sub(r"<!-- TRACE:BEGIN -->\n.*?<!-- TRACE:END -->",
                 f"<!-- TRACE:BEGIN -->\n{trace}\n<!-- TRACE:END -->", T02, flags=re.S)
    F02.write_text(new, encoding="utf-8")
    print("추적 매트릭스를 갱신했다.")
elif not tm:
    err("C17", "02 부록 A에 TRACE 표식이 없음")
elif tm.group(1).strip() != trace.strip():
    err("C17", "02 부록 A 추적 매트릭스가 최신이 아님 → --write-trace 실행")

# ---------------------------------------------------------------- C20 DR 연속성
dr_nums = sorted(int(k[3:]) for k in DR)
if dr_nums != list(range(1, len(dr_nums) + 1)):
    err("C20", f"DR 번호가 연속되지 않음: {dr_nums}")

# ---------------------------------------------------------------- 결과
print("OpenHOI 문서 검사")
print(f"  D {len(D)} · REQ {len(REQ)} · OPEN {len(OPEN)} · AC {len(AC)} · WP {len(WP)} · DR {len(DR)} · P {len(P)}")
by_ms = defaultdict(int)
for ms in req_ms.values():
    by_ms[ms] += 1
print("  마일스톤별 REQ: " + ", ".join(f"{m} {by_ms[m]}" for m in MILESTONES))
print(f"  사용 금지 표현 {len(forbidden)}개 · orch.sh 하위 명령 {sorted(subcmds)} · 템플릿 {len(TEMPLATE_FILES)}개")
for w in warnings:
    print("경고 " + w)
for e in errors:
    print("오류 " + e)
print(f"결과: 오류 {len(errors)} · 경고 {len(warnings)}")
sys.exit(1 if errors else 0)
