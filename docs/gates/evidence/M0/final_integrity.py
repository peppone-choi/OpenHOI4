import pathlib,subprocess,hashlib,json,datetime
root=pathlib.Path.cwd();ev=root/'target/evidence/M0-gate-verify'
tracked=subprocess.check_output(['git','ls-files'],text=True).splitlines();after=[{'path':p,'sha256':hashlib.sha256((root/p).read_bytes()).hexdigest().upper()} for p in tracked]
(ev/'after-tracked-sha256.json').write_text(json.dumps(after,indent=2),encoding='utf8')
before=json.loads((ev/'before-tracked-sha256.json').read_text(encoding='utf-8-sig'));bh={r['path']:r['sha256'] for r in before};ah={r['path']:r['sha256'] for r in after}
head=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip();status=subprocess.check_output(['git','status','--porcelain=v1'],text=True);diff=subprocess.check_output(['git','diff'],text=True);cached=subprocess.check_output(['git','diff','--cached'],text=True)
(ev/'after-head.txt').write_text(head+'\n');(ev/'after-status.txt').write_text(status);(ev/'after-diff.txt').write_text(diff);(ev/'after-cached-diff.txt').write_text(cached)
start=datetime.datetime.fromtimestamp((ev/'before-head.txt').stat().st_mtime,datetime.timezone(datetime.timedelta(hours=9))).isoformat();end=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=9))).isoformat()
result={'start_kst':start,'end_kst':end,'head':head,'head_same':head==(ev/'before-head.txt').read_text(encoding='utf-8-sig').strip(),'tracked_count_before':len(before),'tracked_count_after':len(after),'tracked_list_same':list(bh)==list(ah),'tracked_sha256_same':bh==ah,'status_same':status.strip()==((ev/'before-status.txt').read_text(encoding='utf-8-sig').strip() if (ev/'before-status.txt').exists() else ''),'baseline_status_observation':'Initial git status output was empty; the PowerShell empty pipeline did not create before-status.txt.','status_clean':not status,'diff_clean':not diff,'cached_diff_clean':not cached}
assert all(result[k] for k in ['head_same','tracked_list_same','tracked_sha256_same','status_same','status_clean','diff_clean','cached_diff_clean']),result
(ev/'integrity-final.json').write_text(json.dumps(result,indent=2));print(json.dumps(result,indent=2))
results=json.loads((ev/'local-results.json').read_text(encoding='utf8'));assert len(results)==19 and all(r['exit']==0 for r in results)
summary={'AC-M0-'+str(i).zfill(2):'PASS' for i in range(1,8)};summary.update({'M0_required_REQ_count':14,'rust_tests':46,'compile_fail_doc_tests':2,'client_tests':3,'E2E_tests':35,'exact_display_browsers':5,'remote_CI_jobs':15,'remote_archives_digest_verified':7,'remote_file_comparison':{'files':25,'exact_git_blob':21,'only_newline_different':4},'local_1000_tick_hash':'ff921fd8148e699d','core_probe_hash':'0dd81b8754bcc3f9','integrity':result})
assert '35 passed' in (ev/'e2e.log').read_text(encoding='utf8');assert len(json.loads((ev/'exact-display.json').read_text()))==5
(ev/'gate-summary.json').write_text(json.dumps(summary,indent=2));print('AC-M0-01..07 PASS; integrity PASS')
