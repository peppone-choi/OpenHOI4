"""Budgeted official DEM subset parser tests; no runtime download."""
from pathlib import Path
import unittest
import numpy as np
import world_preview as wp

class DemTests(unittest.TestCase):
    def test_corrupt_tiff_rejected(self):
        with self.assertRaises(ValueError):
            wp.read_dem(b'not a TIFF')

    def test_fixed_official_subset_units_bounds_and_samples(self):
        path=Path(__file__).resolve().parents[2]/'target/evidence/WORLD-PREVIEW/ETOPO2022-himalaya.tif'
        if not path.exists():
            self.skipTest('acquisition evidence not present; no download fallback')
        raster,extent=wp.read_dem(path.read_bytes())
        self.assertEqual(raster.shape,(480,1008))
        self.assertEqual(extent,[74,25,95,35])
        self.assertGreater(float(np.max(raster)),7500)
        self.assertLess(float(np.max(raster)),9000)
        x=int((85.3-74)/(95-74)*1008);y=int((35-27.7)/(35-25)*480)
        self.assertGreater(raster[y,x],900)
        self.assertLess(raster[y,x],2000)

if __name__=='__main__':
    unittest.main()
