"""Cold-cache preparation contract; original Linux native101 is separate evidence."""
import os, tempfile, unittest, json
from pathlib import Path
from run import prepare_metadata,linked_inventory,verify_linked
ROOT=Path(__file__).resolve().parents[1]
LOCK='''version = 4
[[package]]
name = "crunchy"
version = "0.2.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "460fbee9c2c2f33933d720630a6a0bac33ba7053db5344fac858d4b8952d77d5"
'''
def metadata(tree,driver):
    packages=[{'name':name,'source':None,'manifest_path':str(tree/'crates'/name/'Cargo.toml')} for name in ('oh_cli','oh_core','oh_save')]
    packages.append({'name':'oh_bench_driver','source':None,'manifest_path':str(driver/'Cargo.toml')})
    return json.dumps({'packages':packages}).encode()
class Metadata(unittest.TestCase):
    def fixture(self):
        base=Path(os.environ.get('OH_WP25_EVIDENCE_ROOT',ROOT/'target/evidence/WP-25-metadata'))
        base.mkdir(parents=True,exist_ok=True)
        path=Path(tempfile.mkdtemp(prefix='metadata-',dir=base))
        tree=path/'source'; driver=path/'driver'; tree.mkdir();driver.mkdir()
        for p in (tree,driver):
            (p/'Cargo.toml').write_text('[workspace]\n',encoding='utf-8')
            (p/'Cargo.lock').write_text(LOCK,encoding='utf-8')
        return tree,driver
    def test_req_perf_03_cold_cache_locked_fetch_precedes_offline_metadata(self):
        tree,driver=self.fixture(); calls=[]; cache=False
        def command(name,argv,timeout):
            nonlocal cache
            calls.append((name,argv,timeout))
            if argv[1]=='fetch':
                self.assertIn('--locked',argv)
                self.assertNotIn('--offline',argv)
                self.assertEqual(argv[-1],str(tree/'Cargo.toml'))
                cache=True
            elif not cache: raise ValueError('baseline-metadata: native101; crunchy v0.2.4 HTTP blocked by --offline')
            return metadata(tree,driver)
        before=(tree/'Cargo.lock').read_bytes()
        prepare_metadata('baseline',tree,driver,command,{'build_timeout_seconds':1200})
        self.assertEqual([x[1][1] for x in calls],['fetch','metadata'])
        self.assertEqual([x[2] for x in calls],[1200,1200])
        self.assertEqual(before,(tree/'Cargo.lock').read_bytes())
    def test_registry_version_and_checksum_drift_remains_an_error(self):
        for field,value in [('version','0.2.5'),('checksum','0'*64)]:
            tree,driver=self.fixture()
            text=LOCK.replace('0.2.4',value) if field=='version' else LOCK.replace('460fbee9c2c2f33933d720630a6a0bac33ba7053db5344fac858d4b8952d77d5',value)
            (driver/'Cargo.lock').write_text(text,encoding='utf-8')
            with self.assertRaisesRegex(ValueError,'changed source registry resolution'):
                prepare_metadata('current',tree,driver,lambda *args:metadata(tree,driver),{'build_timeout_seconds':1200})
    def test_fetch_failure_is_fatal_and_does_not_try_metadata(self):
        tree,driver=self.fixture();calls=[]
        def command(name,*args):
            calls.append(name)
            raise ValueError(name+': native101')
        with self.assertRaisesRegex(ValueError,'current-fetch'):
            prepare_metadata('current',tree,driver,command,{'build_timeout_seconds':1200})
        self.assertEqual(calls,['current-fetch'])
    def test_source_lock_mutation_is_fatal_before_metadata(self):
        tree,driver=self.fixture();calls=[]
        def command(name,*args):
            calls.append(name)
            (tree/'Cargo.lock').write_text(LOCK+'\n# mutation\n',encoding='utf-8')
            return metadata(tree,driver)
        with self.assertRaisesRegex(ValueError,'source lock changed'):
            prepare_metadata('baseline',tree,driver,command,{'build_timeout_seconds':1200})
        self.assertEqual(calls,['baseline-fetch'])

    def test_manifest_linkage_or_external_path_drift_is_rejected(self):
        tree,driver=self.fixture();m=json.loads(metadata(tree,driver));m['packages'][0]['manifest_path']=str(driver/'other/Cargo.toml')
        with self.assertRaisesRegex(ValueError,'escaped source path dependency'):
            prepare_metadata('current',tree,driver,lambda *args:json.dumps(m).encode(),{'build_timeout_seconds':1200})
        m=json.loads(metadata(tree,driver));m['packages'][0]['manifest_path']=str(tree/'crates/wrong/Cargo.toml')
        with self.assertRaisesRegex(ValueError,'library linkage changed'):
            prepare_metadata('baseline',tree,driver,lambda *args:json.dumps(m).encode(),{'build_timeout_seconds':1200})
    def test_real_library_file_changed_after_linkage_snapshot_is_fatal(self):
        tree,driver=self.fixture();(tree/'rust-toolchain.toml').write_bytes(b'pinned actual toolchain');(driver/'src').mkdir();(driver/'src/main.rs').write_bytes(b'fn main() {}')
        for name in ('oh_cli','oh_core','oh_save'):
            root=tree/'crates'/name;root.mkdir(parents=True);(root/'Cargo.toml').write_bytes(b'actual path manifest');(root/'src').mkdir();(root/'src/lib.rs').write_bytes(b'pub fn actual_library() {}')
        m=json.loads(metadata(tree,driver));expected=linked_inventory(m,tree,driver)
        (tree/'crates/oh_core/src/lib.rs').write_bytes(b'pub fn changed_after_linkage() {}')
        with self.assertRaisesRegex(ValueError,'linked library source identity changed'):verify_linked(m,tree,driver,expected)
if __name__=='__main__': unittest.main(verbosity=2)
