Temporary WP08 DPR diagnostic harness; never integrate into main/product tree.

Scope: exact product source b73f0ce344138d5fb3909eca5d43c786aac49b15;
strict candidate template SHA c9002e018ad536cc48308973a5d3400d69a8925c9732e936bdef30144ae429ac.
The candidate remains a diagnostic target. This branch does not approve or alter
client/e2e-m1/map-dpr.spec.ts, other product files, original CI, schema or game rules.
candidate.patch/expected-fixture.patch preserve the reviewed raw diffs. The first
is against the Node-instrumented original; the second describes a possible later
P06 fixture change and is not applied here.
Tool-directory .gitattributes preserves exact reviewed bytes across Git checkout;
it changes no product paths or security settings. Workflow bytes use LF. Manifest
validation must compare actual staged/committed blobs, not just working-copy files.

Workflow: exact codex/wp08-dpr-diagnostic-m1r2 push and temporary workflow/tool
path filters only. Parent reviews the exact committed files before ONE push.
No workflow_dispatch/input/schedule/PR trigger, repeated push/test retry loop,
isolated follow-up, deploy, tag or additional paid service. Original CI's existing
push triggers remain unchanged and can run independently; distinguish results.

One fixed experiment on a separate exact source checkout:
19471 original M0 config/three projects36 -> full M1 original/probe82;
19472 original M0 config/three projects36 -> full M1 strict candidate82.
Original workers, browser options/GPU preferences, six Chromium DPR values,
full PNG decode/capture, rawGL, old contexts, camera, authority, lifetime,
cardinality and final cleanup remain. Per-test30s/expect5s/retry0/skip0 unchanged.
finally evidence and context.close remain in the body budget. Node-only profile
records phases/client-driver requests; it does not measure every inner CDP call,
wrap GPU APIs, force a worker restart or prove a Linux cause from Windows timing.

Job-level and every helper Git read use GIT_OPTIONAL_LOCKS=0 to avoid optional
index refresh. No Git configuration, sandbox, approval/network setting is edited.
Before/after prep and execution compare HEAD, actual raw index bytes/SHA,
canonical index entries, tracked paths and each file SHA, staged/unstaged diff
and status. Run start must equal prep after-state. Per-arm states are preserved.

prep_linux.py compares actual --list output for original/probe/candidate:
ordered project, normalized file basename, entire title including nested groups,
all82 entries and the actual discovered eight-file set. File/line path adaptation
does not drop fields. Raw lists and structured proof are retained.

run_linux.py reads /proc/PID/exe bytes and compares byte count/SHA/content against
the exact built source executable, after checking owning listener PID/exe path.
Served JS bytes are compared against the corresponding safe source client/dist
asset path. M0 and both M1 arms require these identities and actual case counts.
Any mismatch makes diagnostic result fail; identity mismatches are not retried.
Before server readiness only the own-port identity watcher waits for startup;
it never repeats browser test runs. Failure logs/artifacts are preserved.

Preparation commands in source checkout after committed toolkit is copied under
ignored target/wp08-dpr-ci2:
python target/wp08-dpr-ci2/prep_linux.py
python target/wp08-dpr-ci2/run_linux.py

The source checkpoint is not a harness checkout. Runtime experiment is Linux-only.
Local preparation/collection and stdlib negative guards are separate evidence;
no Linux run is claimed by this commit. Normal latest main CI and later fresh
independent validation remain required. Stop after reporting this local commit;
the worker must not push or dispatch it.

Sources read 2026-10-07 KST:
https://git-scm.com/docs/git#Documentation/git.txt-codeGITOPTIONALLOCKScode
https://docs.github.com/en/actions/how-tos/manage-workflow-runs/manually-run-a-workflow
https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#push
https://playwright.dev/docs/test-timeouts
