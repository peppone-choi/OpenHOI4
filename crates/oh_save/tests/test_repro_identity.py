"""Exact committed F01 bytes, not merely an unnormalized implementation checkout."""
import hashlib,io,json,subprocess,tarfile,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
PREFIX='tests/repro/WP-14-M2-r3-restore-stage/packs/testland'
class ReproIdentity(unittest.TestCase):
    def test_original_f01_complete_git_archive_matches_sealed_manifest(self):
        expected=json.loads((ROOT/'tests/repro/WP-14-M2-r3-restore-stage/INPUT_MANIFEST.json').read_text())['files']
        archive=subprocess.check_output(['git','--no-optional-locks','archive','--format=tar','HEAD',PREFIX],cwd=ROOT)
        out=ROOT/'target/evidence/WP-14-M2-r3-P06-2/git-archive';out.mkdir(parents=True,exist_ok=True)
        head=subprocess.check_output(['git','--no-optional-locks','rev-parse','HEAD'],cwd=ROOT,text=True).strip();(out/(head+'.tar')).write_bytes(archive)
        actual=[]
        with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
            for member in tar.getmembers():
                self.assertTrue(member.isdir() or member.isfile())
                if member.isfile():
                    data=tar.extractfile(member).read();actual.append(dict(path=member.name.removeprefix(PREFIX+'/'),bytes=len(data),sha256=hashlib.sha256(data).hexdigest()))
        actual.sort(key=lambda row:row['path']);expected.sort(key=lambda row:row['path'])
        (out/(head+'.json')).write_text(json.dumps(dict(head=head,files=actual),indent=2),encoding='utf-8')
        self.assertEqual(actual,expected,'fresh Git archive must preserve the full original pack, including the two CRLF fixtures')
if __name__=='__main__':unittest.main(verbosity=2)
