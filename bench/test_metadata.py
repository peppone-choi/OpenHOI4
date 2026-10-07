"""Cold-cache preparation contract; original Linux native101 is separate evidence."""
import os, tempfile, unittest
from pathlib import Path
from run import prepare_metadata
ROOT=Path(__file__).resolve().parents[1]
LOCK='''version = 4
[[package]]
name = "crunchy"
version = "0.2.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "460fbee9c2c2f33933d720630a6a0bac33ba7053db5344fac858d4b8952d77d5"
'''
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
            return b'{}'
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
                prepare_metadata('current',tree,driver,lambda *args:b'{}',{'build_timeout_seconds':1200})
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
            return b'{}'
        with self.assertRaisesRegex(ValueError,'source lock changed'):
            prepare_metadata('baseline',tree,driver,command,{'build_timeout_seconds':1200})
        self.assertEqual(calls,['baseline-fetch'])
if __name__=='__main__': unittest.main(verbosity=2)
