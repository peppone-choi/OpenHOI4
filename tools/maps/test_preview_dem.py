"""Budgeted official DEM subset parser tests; no runtime download."""
from pathlib import Path
import unittest
import numpy as np
import world_preview as wp

class DemTests(unittest.TestCase):
    def test_relief_and_river_geodesic_allocation_changes_shape(self):
        kind=np.ones((20,30),dtype=np.uint8);seeds=np.array([[5,5],[5,24]])
        river=np.zeros_like(kind,dtype=bool);river[:,13]=True;river[10:,13]=False;river[10:,18]=True
        elevation=np.zeros_like(kind,dtype=float);elevation[7:14,16:20]=4000
        options={'river_crossing_penalty':20,'relief_meters':500,'ridge_penalty':2,'ridge_window':5}
        plain=wp.geodesic_labels(kind,1,seeds,np.zeros_like(river),np.full_like(elevation,np.nan),options)
        shaped=wp.geodesic_labels(kind,1,seeds,river,elevation,options)
        self.assertGreater(np.count_nonzero(plain!=shaped),20)
        self.assertEqual(shaped[5,5],0);self.assertEqual(shaped[5,24],1)
        self.assertTrue(np.all(shaped>=0))

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
