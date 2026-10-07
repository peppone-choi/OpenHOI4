"""P06-4 independent ordinal oracle and allocation-cost regressions."""
import unittest
from unittest.mock import patch
import numpy as np
import world_preview as wp


def floodfill_ordinals(raw):
    """Small independent 4-neighbour traversal, seed order then row-major start."""
    h,w=raw.shape;seen=np.zeros(raw.shape,bool);result=np.zeros(raw.shape,np.uint16);ordinal=0
    for seed in sorted(int(v) for v in np.unique(raw)):
        for y in range(h):
            for x in range(w):
                if seen[y,x] or raw[y,x]!=seed:continue
                if ordinal>=65535:raise ValueError('u16 ID overflow')
                todo=[(y,x)];seen[y,x]=True
                while todo:
                    yy,xx=todo.pop();result[yy,xx]=ordinal
                    for dy,dx in [(-1,0),(1,0),(0,-1),(0,1)]:
                        ny,nx=yy+dy,xx+dx
                        if 0<=ny<h and 0<=nx<w and not seen[ny,nx] and raw[ny,nx]==seed:
                            seen[ny,nx]=True;todo.append((ny,nx))
                ordinal+=1
    return result


class ComparisonParts(np.ndarray):
    def __array_finalize__(self,obj):self.cost=getattr(obj,'cost',None)
    def __eq__(self,value):
        self.cost.append(self.size);return np.asarray(super().__eq__(value))
    def __ne__(self,value):
        self.cost.append(self.size);return np.asarray(super().__ne__(value))


class ComponentCostTests(unittest.TestCase):
    def test_sparse_seed_slots_far_pieces_keep_component_and_seed_order(self):
        raw=np.array([[9,9,2,9],[2,9,2,9],[9,2,9,9]],dtype=np.int32)
        expected=floodfill_ordinals(raw)
        self.assertEqual(expected.tolist(),[[3,3,0,4],[1,3,0,4],[5,2,4,4]])
        self.assertTrue(np.array_equal(wp.connected_ids(raw),expected))
        self.assertEqual(wp.connected_ids(raw).dtype,np.dtype('uint16'))

    def test_unsigned_signed_and_noncontiguous_inputs_keep_background_assignments(self):
        base=np.array([[0,8,0,2,8,0],[8,8,2,2,8,2],[0,2,8,0,2,0],[2,8,2,8,0,2]])
        for dtype in [np.uint8,np.uint16,np.int16,np.int32,np.int64]:
            for raw in [base.astype(dtype),base.astype(dtype)[::2,::-1],base.astype(dtype).T]:
                with self.subTest(dtype=dtype,strides=raw.strides):
                    self.assertTrue(np.array_equal(wp.connected_ids(raw),floodfill_ordinals(raw)))

    def test_component_relabel_comparison_work_does_not_multiply_by_piece_count(self):
        raw=np.full((24,40),7,dtype=np.int32);raw[::3,::3]=0
        expected=floodfill_ordinals(raw);original_label=wp.ndimage.label;cost=[];labelled_box_pixels=[]
        def label(mask):
            parts,count=original_label(mask);parts=parts.view(ComparisonParts);parts.cost=cost
            labelled_box_pixels.append(mask.size);return parts,count
        with patch.object(wp.ndimage,'label',side_effect=label):actual=wp.connected_ids(raw)
        self.assertTrue(np.array_equal(actual,expected))
        # One background mask per seed bbox; disconnected-piece count must not
        # cause the same bbox to be compared/painted once per component.
        self.assertLessEqual(sum(cost),sum(labelled_box_pixels))

    def test_65535_ids_allowed_and_65536_rejected_without_wrapped_ordinals(self):
        raw=np.arange(65535,dtype=np.int32).reshape(255,257)
        actual=wp.connected_ids(raw)
        self.assertEqual(int(actual.min()),0);self.assertEqual(int(actual.max()),65534)
        self.assertTrue(np.array_equal(actual,raw.astype(np.uint16)))
        with self.assertRaisesRegex(ValueError,'u16 ID overflow'):
            wp.connected_ids(np.arange(65536,dtype=np.int32).reshape(256,256))

    def test_first_component_cell_is_row_major_for_strided_boxes_and_dtypes(self):
        base=np.array([[2,2,9,4],[4,9,4,4],[2,9,2,4],[9,2,2,4]])
        for dtype in [np.uint16,np.int32,np.int64]:
            for index in [base.astype(dtype),base.astype(dtype)[::-1,::2],base.astype(dtype).T]:
                box=(slice(0,index.shape[0]),slice(0,index.shape[1]))
                for dense in np.unique(index):
                    expected=tuple(map(int,np.argwhere(index==dense)[0]))
                    self.assertEqual(wp.first_component_cell(index,box,int(dense)),expected)
                box=(slice(1,index.shape[0]),slice(1,index.shape[1]))
                for dense in np.unique(index[box]):
                    y,x=map(int,np.argwhere(index[box]==dense)[0])
                    self.assertEqual(wp.first_component_cell(index,box,int(dense)),(y+1,x+1))

    def test_repair_first_cell_does_not_allocate_all_component_coordinates(self):
        raw=np.array([[0,0,0,1,1],[0,1,0,1,1],[0,0,0,2,2]],dtype=np.int32)
        expected=raw.copy();expected[1,1]=0;kind=np.ones(raw.shape,np.uint8);urban=np.zeros(raw.shape,bool)
        with patch.object(wp.np,'argwhere',side_effect=AssertionError('all coordinates allocated for first cell')):
            actual,report=wp.repair_internal_fragments(raw.copy(),kind,urban,urban,1)
        self.assertTrue(np.array_equal(actual,expected));self.assertEqual(report['reassigned_components'],1)
        self.assertEqual(report['records'][0]['cells'],[[1,1]])
        self.assertEqual(report['records'][0]['target_raw_seed'],0)
        self.assertEqual(report['records'][0]['shared_edges'],4)


if __name__=='__main__':unittest.main()
