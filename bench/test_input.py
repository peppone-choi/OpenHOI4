"""Actual filesystem attacks against immutable full-pack measurements."""
import os, tempfile, unittest, hashlib, subprocess
from pathlib import Path
from run import pack_inventory, verify_pack, native_sample

class ImmutableInput(unittest.TestCase):
    def fixture(self):
        root=Path(tempfile.mkdtemp(dir=Path(__file__).resolve().parents[1]/'target'))
        pack=root/'prior'/'data'/'packs'/'testland'; pack.mkdir(parents=True)
        (pack/'manifest.toml').write_bytes(b'full prior manifest')
        (pack/'scenarios').mkdir(); (pack/'scenarios'/'m1.toml').write_bytes(b'prior m1')
        (pack/'unrelated').mkdir(); (pack/'unrelated'/'other.bin').write_bytes(b'also in full pack')
        binary=root/'driver'; binary.write_bytes(b'actual binary provenance')
        return pack,binary
    def test_full_prior_path_is_shared_and_current_content_is_irrelevant(self):
        pack,binary=self.fixture(); expected=pack_inventory(pack)
        current=pack.parents[3]/'current';current.mkdir()
        (current/'m2.toml').write_bytes(b'new content')
        calls=[]
        def command(name,argv): calls.append(argv); return b'{}'
        for label in ['baseline','current']:
            native_sample(label,binary,pack,expected,{'scenario':'m1','seed':1000},2400,command)
        self.assertEqual(calls[0][1],calls[1][1]); self.assertEqual(calls[0][1],str(pack))
        self.assertEqual([f['path'] for f in expected['files']],['manifest.toml','scenarios/m1.toml','unrelated/other.bin'])
    def test_extra_missing_corrupt_and_empty_directory_change_are_rejected(self):
        for attack in ['extra','missing','corrupt','directory']:
            pack,_=self.fixture(); expected=pack_inventory(pack)
            if attack=='extra': (pack/'new').write_bytes(b'new')
            if attack=='missing': (pack/'unrelated'/'other.bin').unlink()
            if attack=='corrupt': (pack/'manifest.toml').write_bytes(b'FULL prior manifest')
            if attack=='directory': (pack/'extra-empty').mkdir()
            with self.assertRaisesRegex(ValueError,'pack identity changed'): verify_pack(pack,expected)
    def test_mutation_during_real_invocation_is_fatal(self):
        pack,binary=self.fixture(); expected=pack_inventory(pack)
        def command(*args): (pack/'manifest.toml').write_bytes(b'changed during load'); return b'{}'
        with self.assertRaisesRegex(ValueError,'pack identity changed'):
            native_sample('current-warmup',binary,pack,expected,{'scenario':'m1','seed':1000},2400,command)
    def test_link_escape_is_rejected(self):
        pack,binary=self.fixture()
        try: (pack/'escape').symlink_to(binary)
        except OSError as error: self.skipTest('OS denies symlink fixture: '+str(error))
        with self.assertRaisesRegex(ValueError,'nonregular'): pack_inventory(pack)
    def test_pack_root_link_is_rejected(self):
        pack,_=self.fixture(); alias=pack.parent/'alias'
        try: alias.symlink_to(pack,target_is_directory=True)
        except OSError as error: self.skipTest('OS denies symlink fixture: '+str(error))
        with self.assertRaisesRegex(ValueError,'nonregular'): pack_inventory(alias)

    def test_binary_changed_after_linkage_is_rejected_before_native(self):
        pack,binary=self.fixture();expected=pack_inventory(pack);digest=hashlib.sha256(binary.read_bytes()).hexdigest();binary.write_bytes(b'changed after real binary linkage');calls=[]
        with self.assertRaisesRegex(ValueError,'binary identity changed'):
            native_sample('current',binary,pack,expected,{'scenario':'m1','seed':1000},2400,lambda *args:calls.append(args),digest)
        self.assertFalse(calls)
    @unittest.skipUnless(os.name=='nt','Windows reparse-point fixture')
    def test_actual_windows_junction_root_is_rejected(self):
        pack,_=self.fixture();alias=pack.parent/'junction'
        result=subprocess.run(['cmd','/c','mklink','/J',str(alias),str(pack)],capture_output=True)
        self.assertEqual(result.returncode,0,result.stderr.decode(errors='replace'))
        with self.assertRaisesRegex(ValueError,'nonregular'):pack_inventory(alias)
if __name__=='__main__': unittest.main(verbosity=2)
