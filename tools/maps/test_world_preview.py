"""Actual offline/provenance/topology regressions for the preview generator."""
import json
from pathlib import Path
import tempfile
import unittest
import numpy as np
import world_preview as wp


class PreviewTests(unittest.TestCase):
    def test_missing_source_fails_without_network(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'source-manifest.json').write_text(json.dumps({'sources': [{'file':'missing.zip','sha256':'0'*64}]}))
            with self.assertRaises(FileNotFoundError):
                wp.verify_sources(root)

    def test_hash_mismatch_fails(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'bad.zip').write_bytes(b'not-the-fixed-source')
            (root / 'source-manifest.json').write_text(json.dumps({'sources':[{'file':'bad.zip','sha256':'0'*64}]}))
            with self.assertRaisesRegex(ValueError, 'hash'):
                wp.verify_sources(root)

    def test_no_path_escape(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'source-manifest.json').write_text(json.dumps({'sources':[{'file':'../escape','sha256':'0'*64}]}))
            with self.assertRaises(ValueError):
                wp.verify_sources(root)

    def test_split_disconnected_island_and_sorted_adjacency(self):
        raw = np.array([[0,0,1,0],[0,1,1,0]], dtype=np.int32)
        index = wp.connected_ids(raw)
        self.assertNotEqual(index[0,0], index[0,3])
        self.assertEqual(index[0,3], index[1,3])
        pairs = wp.adjacency(index)
        self.assertEqual(pairs, sorted(set(tuple(p) for p in pairs)))
        self.assertIn(tuple(sorted((int(index[0,0])+1,int(index[0,3])+1))), pairs)

    def test_polygon_hole_and_coordinates(self):
        polygon = [[(-20,20),(20,20),(20,-20),(-20,-20),(-20,20)], [(-5,5),(-5,-5),(5,-5),(5,5),(-5,5)]]
        mask = wp.rasterize([polygon], 360,180)
        self.assertFalse(mask[90,180])
        self.assertTrue(mask[80,190])
        self.assertFalse(mask[20,20])

    def test_little_endian_dense_ids(self):
        index=np.array([[0,256,65535]],dtype=np.uint16)
        self.assertEqual(wp.index_bytes(index), bytes([0,0,0,1,255,255]))

if __name__ == '__main__':
    unittest.main()
