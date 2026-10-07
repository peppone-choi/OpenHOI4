"""P06-3 coverage/footprint/sea valid-case tests before implementation."""
import unittest
import numpy as np
from scipy import ndimage
import world_preview as wp

class CoverageTests(unittest.TestCase):
    def test_large_disconnected_land_is_guaranteed_seeds_despite_weight_bias(self):
        mask=np.zeros((10,22),bool);mask[2:7,2:7]=True;mask[2:7,15:20]=True
        weights=np.where(mask,np.where(np.indices(mask.shape)[1]<10,99.,1.),0)
        config={'large_component_min_area_km2':1000,'large_component_min_seeds':3,'max_seed_area_km2':2000,'seed_spacing_km':0,'seed_oversample':4,'earth_radius_km':6371.0088}
        seeds,report=wp.coverage_seeds(mask,weights,8,np.random.default_rng(32),np.full(10,200.),config)
        labels,_=ndimage.label(mask);counts=np.bincount(labels[seeds[:,0],seeds[:,1]])
        self.assertEqual(len(seeds),8);self.assertGreaterEqual(int(counts[1]),3);self.assertGreaterEqual(int(counts[2]),3)
        self.assertEqual(report['unseeded_large_components'],0)

    def test_impossible_coverage_budget_fails_without_dropping_a_component(self):
        mask=np.array([[1,1,0,1,1],[1,1,0,1,1]],dtype=bool)
        config={'large_component_min_area_km2':1,'large_component_min_seeds':3,'max_seed_area_km2':2000,'seed_spacing_km':0,'seed_oversample':4,'earth_radius_km':6371.0088}
        with self.assertRaisesRegex(ValueError,'coverage budget'):
            wp.coverage_seeds(mask,mask.astype(float),5,np.random.default_rng(1),np.ones(2),config)

    def test_sea_coastal_spacing_uses_sphere_across_dateline(self):
        candidates=np.array([[90,0],[90,359]],dtype=np.int32)
        picked=wp.spaced_samples(candidates,360,180,6371.0088,300,2)
        self.assertEqual(len(picked),1)

    def test_new_source_kind_without_coarse_anchor_gets_a_valid_independent_id(self):
        mapped=wp.refine_coast(np.zeros((1,2),dtype=np.int32),np.ones((1,2),dtype=np.uint8),np.array([[1,2,1,2]],dtype=np.uint8))
        self.assertEqual(mapped[0,0],0);self.assertEqual(mapped[0,2],0)
        self.assertNotEqual(mapped[0,1],0);self.assertNotEqual(mapped[0,3],0)
        index=wp.connected_ids(mapped)
        self.assertEqual(index.tolist(),[[0,2,1,3]])
        self.assertEqual(len(np.unique(index)),4)
        self.assertEqual(wp.adjacency(index),[(1,3),(1,4),(2,3),(2,4)])

    def test_real_footprint_is_separate_from_nonurban_and_never_overwrites_water(self):
        raw=np.zeros((4,8),dtype=np.int32);kind=np.ones_like(raw,dtype=np.uint8);kind[1,4]=2;raw[1,4]=5
        urban=np.zeros_like(raw,dtype=bool);urban[1:3,2:5]=True
        definitions={'quality':{'earth_radius_km':1},'urban':{'minimum_area_km2':0,'target_area_km2':100,'maximum_seeds':32},'partition':{'river_crossing_penalty':12,'relief_meters':500,'ridge_penalty':.7,'ridge_window':3}}
        mapped,report=wp.partition_urban(raw,kind,urban,np.zeros_like(urban),[],np.empty((0,2)),definitions,1)
        self.assertEqual(mapped[1,4],5);self.assertEqual(mapped[0,0],0)
        self.assertTrue(np.all(mapped[urban&(kind==1)]!=0));self.assertGreater(report['partitioned_components'],0)

    def test_point_without_footprint_never_becomes_urban(self):
        raw=np.zeros((4,8),dtype=np.int32);kind=np.ones_like(raw,dtype=np.uint8)
        definitions={'quality':{'earth_radius_km':1},'urban':{'minimum_area_km2':0,'target_area_km2':100,'maximum_seeds':32},'partition':{'river_crossing_penalty':12,'relief_meters':500,'ridge_penalty':.7,'ridge_window':3}}
        mapped,report=wp.partition_urban(raw,kind,np.zeros_like(raw,dtype=bool),np.zeros_like(raw,dtype=bool),[],np.array([[0.,0.]]),definitions,1)
        self.assertTrue(np.array_equal(mapped,raw));self.assertEqual(report['partitioned_components'],0)

if __name__=='__main__':unittest.main()
