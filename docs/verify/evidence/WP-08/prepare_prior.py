from pathlib import Path
import difflib, json
root=Path(__file__).resolve().parent
changes={}
for name in ['cors-valid-redirect.cjs','cors-valid-index-redirect.cjs']:
 original=(root/'prior-attempt'/name).read_text(encoding='utf-8')
 adjusted=original.replace('19412','19415')
 (root/name).write_text(adjusted,encoding='utf-8')
 changes[name]=''.join(difflib.unified_diff(original.splitlines(True),adjusted.splitlines(True),fromfile='preserved-original',tofile='execution-copy'))
(root/'prior-adjustments.json').write_text(json.dumps({'only_change':'port 19412 -> 19415; original historical commit label preserved','testedCommit':'87c1a99a30de25732376d628958c68d37ce64cda','diffs':changes},indent=2),encoding='utf-8')
