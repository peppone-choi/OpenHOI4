"""P06-2 independent contracts before implementation."""
import unittest
import numpy as np
import world_preview as wp
import bounded_bake

class QualityTests(unittest.TestCase):
    def test_coast_reassignment_never_uses_an_anchor_reclassified_by_other_kind(self):
        coarse=np.array([[0,1]],dtype=np.int32)
        kinds=np.array([[0,1]],dtype=np.uint8)
        fine=np.array([[1,1,0,1]],dtype=np.uint8)
        self.assertEqual(wp.refine_coast(coarse,kinds,fine).tolist(),[[1,1,0,1]])
    def test_owned_bake_budget_rejects_time_or_either_memory_measure(self):
        limits={'bake_seconds':180,'bake_memory_bytes':300}
        self.assertIsNone(bounded_bake.budget_reason(179,299,299,limits))
        self.assertEqual(bounded_bake.budget_reason(181,0,0,limits),'time budget exceeded')
        self.assertEqual(bounded_bake.budget_reason(1,301,0,limits),'memory budget exceeded')
        self.assertEqual(bounded_bake.budget_reason(1,0,301,limits),'memory budget exceeded')
    def test_edgeless_islands_are_valid_and_seedless_island_retains_fallback(self):
        kind=np.array([[1,0],[0,1]],dtype=np.uint8)
        options={'river_crossing_penalty':12,'relief_meters':500,'ridge_penalty':.7,'ridge_window':3}
        actual=wp.geodesic_labels(kind,1,np.array([[0,0]]),np.zeros_like(kind,dtype=bool),np.full(kind.shape,np.nan),options)
        self.assertEqual(actual.tolist(),[[0,-1],[-1,-1]])
    def test_extent_pixel_centres_and_no_region_wrap(self):
        # Known geographic rectangle; empty far-away polygon must be harmless.
        polygon=[[(126,36),(128,36),(128,38),(126,38),(126,36)]]
        distant=[[(-100,0),(-99,0),(-99,1),(-100,1),(-100,0)]]
        mask=wp.rasterize([polygon,distant],100,110,extent=[122,32,132,43])
        self.assertTrue(mask[60,50]);self.assertFalse(mask[0,0]);self.assertEqual(int(mask.sum()),400)
        kind=np.ones((10,20),dtype=np.uint8);kind[:,10]=0
        labels=wp.geodesic_labels(kind,1,np.array([[5,2]]),np.zeros_like(kind,dtype=bool),np.full(kind.shape,np.nan),{'river_crossing_penalty':12,'relief_meters':500,'ridge_penalty':.7,'ridge_window':3},extent=[122,32,132,43])
        self.assertTrue(np.all(labels[:,11:]<0))

    def test_u16_id_overflow_rejected_before_output(self):
        raw=np.arange(65536,dtype=np.int32).reshape(256,256)
        with self.assertRaisesRegex(ValueError,'u16'):
            wp.connected_ids(raw)

    def test_city_distance_uses_actual_coordinates_and_physical_extent(self):
        distances=wp.city_distances(np.array([[127.,37.]]),[126,36,128,38],100,100,6371.0088)
        self.assertLess(float(distances[49,49]),2)
        self.assertGreater(float(distances[0,0]),100)
        self.assertTrue(np.isfinite(distances).all())

    def test_new_coast_cells_keep_kind_and_region_boundary_ids(self):
        coarse=np.array([[0,0,1],[0,2,1]],dtype=np.int32)
        kinds=np.array([[0,0,1],[0,1,1]],dtype=np.uint8)
        fine=np.repeat(np.repeat(kinds,2,0),2,1);fine[1,1]=1
        mapped=wp.refine_coast(coarse,kinds,fine)
        label_kind={0:0,1:1,2:1}
        self.assertEqual(label_kind[int(mapped[1,1])],1)
        self.assertTrue(np.all(mapped[:,4:]==1))

if __name__=='__main__':unittest.main()
