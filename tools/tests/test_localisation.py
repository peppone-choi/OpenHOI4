import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('checker', ROOT/'tools/check_localisation.py')
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)


class Localisation(unittest.TestCase):
    def test_static_call_missing_in_both_is_not_hidden(self):
        with tempfile.TemporaryDirectory(dir=ROOT/'target') as directory:
            p = Path(directory)
            (p/'test.tsx').write_text("const label = t('missing-key');", encoding='utf-8')
            uses = checker.collect(p, [])
            self.assertTrue(any(u['key']=='missing-key' and u['required'] for u in uses))
            ko = p/'ko.ftl'
            en = p/'en.ftl'
            ko.write_text('existing = Existing\n', encoding='utf-8')
            en.write_text('existing = Existing\n', encoding='utf-8')
            payload = p/'uses.json'
            payload.write_text(json.dumps(uses), encoding='utf-8')
            result = subprocess.run(['cargo','run','-p','oh_data','--locked','--example','catalog_check','--',
                                     str(ko),str(en),str(payload)],cwd=ROOT,text=True,encoding='utf-8',capture_output=True)
            self.assertEqual(result.returncode,1,result.stderr)
            self.assertIn('missing ko message value missing-key',result.stderr)
            self.assertIn('test.tsx:1:18',result.stderr)

    def test_generated_dto_keys_require_registry_names(self):
        with tempfile.TemporaryDirectory(dir=ROOT/'target') as directory:
            p = Path(directory)
            source=p/'src'
            source.mkdir()
            pack=p/'pack'
            (pack/'common').mkdir(parents=True)
            (pack/'common/resources.toml').write_text('[[resource]]\nid="new_resource"\n',encoding='utf-8')
            uses=checker.collect(source,[pack])
            self.assertEqual([u['key'] for u in uses],['resource-new_resource'])
            self.assertTrue(uses[0]['required'])

    def test_frozen_pack_is_exact_original_tree(self):
        source=json.loads((ROOT/'crates/oh_save/tests/fixtures/m1-pack-v1.source.json').read_text())
        prefix=source['source_path']+'/'
        original=subprocess.check_output(['git','ls-tree','-r','--name-only',source['source_commit'],'--',source['source_path']],cwd=ROOT,text=True).splitlines()
        self.assertEqual({p.removeprefix(prefix) for p in original},{r['path'] for r in source['files']})
        import hashlib
        fixture=ROOT/'crates/oh_save/tests/fixtures/m1-pack-v1'
        self.assertEqual({p.relative_to(fixture).as_posix() for p in fixture.rglob('*') if p.is_file()}, {r['path'] for r in source['files']})
        for r in source['files']:
            raw=(fixture/r['path']).read_bytes()
            original=subprocess.check_output(['git','show',source['source_commit']+':'+prefix+r['path']],cwd=ROOT)
            self.assertEqual(raw,original,r['path'])
            self.assertEqual(hashlib.sha256(raw).hexdigest(),r['sha256'])


if __name__=='__main__':
    unittest.main()
