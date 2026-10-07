import unittest
from compare import regression, compare_samples, validate_sample
import copy
class Boundary(unittest.TestCase):
    def test_req_perf_03_exact_boundary(self):
        self.assertFalse(regression([100,100],[115,115],15))
        self.assertTrue(regression([100,100],[116,116],15))
        self.assertFalse(regression([100,100],[116,115],15))
        self.assertFalse(regression([10**100,10**100],[115*10**98,115*10**98],15))
        for bad in [0, -1, None, float('inf'), float('nan'), 1.1, True]:
            with self.assertRaises(ValueError): regression([bad,100],[116,116],15)
    def test_full_native_evidence_rejection(self):
        # Independent known FNV1a64 for one zero byte.
        policy={'steps':100,'seed':1,'scenario':'m1','threshold_percent':15}
        sample={'elapsed_ns':100,'steps':100,'tick':100,'seed':1,'scenario':'m1','ended':False,'dto':{'state':1},'canonical_hex':'00','hash':'af63bd4c8601b7df'}
        self.assertFalse(compare_samples([sample,sample],[sample,sample],policy))
        for field,value in [('elapsed_ns',0),('elapsed_ns',float('nan')),('elapsed_ns',None),('tick',99),('steps',0),('seed',2),('scenario','wrong'),('ended',True),('dto',None),('canonical_hex','xx'),('hash','0'*16)]:
            with self.subTest(field=field,value=value):
                broken=copy.deepcopy(sample); broken[field]=value
                with self.assertRaises((ValueError,TypeError)): validate_sample(broken,policy)
        broken=copy.deepcopy(sample); broken['dto']={'state':2}
        with self.assertRaises(ValueError): compare_samples([sample,sample],[sample,broken],policy)
        with self.assertRaises(ValueError): compare_samples([sample],[sample,sample],policy)
if __name__ == '__main__': unittest.main(verbosity=2)
