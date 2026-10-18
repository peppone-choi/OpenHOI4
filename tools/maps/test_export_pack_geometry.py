"""Small, synthetic contracts; real-world load_map runs only on hosted CI."""
import array
import hashlib
import json
from pathlib import Path
import tempfile
import unittest

import export_pack_geometry as geometry


class GeometryTests(unittest.TestCase):
    def test_same_kind_then_shared_border_then_small_id(self):
        pixels = array.array('H', [2, 2, 3, 3, 2, 1, 3, 3])
        kinds = {1: 'land', 2: 'sea', 3: 'land'}
        result, merges = geometry.absorb(pixels, 4, 2, kinds)
        self.assertEqual(result[5], 3)  # same kind beats two sea borders
        self.assertEqual(merges, [{'id': 1, 'target': 3, 'old_kind': 'land',
                                   'new_kind': 'land', 'pixel': [1, 1]}])
        kinds[3] = 'sea'
        result, _ = geometry.absorb(pixels, 4, 2, kinds)
        self.assertEqual(result[5], 2)  # shared-border tie => smaller ID
        pixels = array.array('H', [2, 2, 3, 3, 2, 1, 2, 3])
        result, _ = geometry.absorb(pixels, 4, 2, kinds)
        self.assertEqual(result[5], 2)  # three shared borders beat one

    def test_wrap_coastal_rgb_and_two_to_eight_pixels_unchanged(self):
        pixels = array.array('H', [1, 1, 2, 2, 3, 3, 3, 3] * 4)
        kinds = {1: 'land', 2: 'lake', 3: 'sea'}
        result, merges = geometry.absorb(pixels, 8, 4, kinds)
        self.assertEqual(result, pixels)
        self.assertEqual(merges, [])
        self.assertIn((1, 3), geometry.edges(result, 8, 4))
        self.assertEqual(geometry.coastal_ids(result, 8, 4, kinds), {1})
        colors = {geometry.rgb(i) for i in range(1, 65536)}
        self.assertEqual(len(colors), 65535)
        for i in (1, 255, 256, 24889, 65535):
            r, g, b = geometry.rgb(i)
            self.assertEqual((r << 8) | g, i)
            self.assertEqual(b, 0)

    def test_no_candidate_fails_without_singleton_chain(self):
        with self.assertRaisesRegex(ValueError, 'singleton 1.*no original'):
            geometry.absorb(array.array('H', [1, 2, 3, 4]), 2, 2,
                            {i: 'land' for i in range(1, 5)})
        # 1 touches singleton 2, but cannot chain through it to province 3.
        pixels = array.array('H', [3, 3, 4, 3, 3, 3, 2, 1, 5, 3, 3, 3, 6, 3, 3])
        with self.assertRaisesRegex(ValueError, 'singleton 1.*no original'):
            geometry.absorb(pixels, 5, 3, {i: 'sea' for i in range(1, 7)})
        self.assertEqual(pixels[7], 1)  # no partial input mutation

    def fixture(self, root):
        pixels = array.array('H', [1, 1, 2, 2, 3, 3, 3, 3] * 4)
        raw = b''.join(i.to_bytes(2, 'little') for i in pixels)
        (root / 'index.bin').write_bytes(raw)
        provinces = [{'id': i, 'kind': k, 'pixels': pixels.count(i)}
                     for i, k in [(1, 'land'), (2, 'lake'), (3, 'sea')]]
        (root / 'provinces.json').write_text(json.dumps(provinces))
        (root / 'adjacency.json').write_text(json.dumps([[1, 2], [1, 3], [2, 3]]))
        hashes = {name: hashlib.sha256((root / name).read_bytes()).hexdigest()
                  for name in ('index.bin', 'provinces.json', 'adjacency.json')}
        (root / 'metadata.json').write_text(json.dumps({
            'schema': 'world-preview-v1', 'width': 8, 'height': 4,
            'byte_order': 'u16-le', 'extent': [-180, -90, 180, 90],
            'hashes': hashes}))

    def test_hash_tampering_and_deterministic_files(self):
        terrain = {'land': 'plains', 'sea': 'ocean', 'lake': 'inland_water'}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.fixture(root)
            geometry.export(root, root / 'a', terrain)
            geometry.export(root, root / 'b', terrain)
            a = {p.name: p.read_bytes() for p in (root / 'a').iterdir()}
            b = {p.name: p.read_bytes() for p in (root / 'b').iterdir()}
            self.assertEqual(a, b)
            for name in ('index.bin', 'provinces.json', 'adjacency.json'):
                original = (root / name).read_bytes()
                (root / name).write_bytes(original + b' ')
                with self.assertRaisesRegex(ValueError, name.replace('.', r'\.')):
                    geometry.export(root, root / 'bad', terrain)
                self.assertFalse((root / 'bad').exists())
                (root / name).write_bytes(original)

    def test_metadata_trust_scope_and_terrain_required(self):
        with self.assertRaises(ValueError):
            geometry.parse_terrain('land=plains,sea=ocean')
        with self.assertRaises(ValueError):
            geometry.parse_terrain('land=plains,sea=ocean,lake=bad/id')
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.fixture(root)
            metadata = json.loads((root / 'metadata.json').read_text())
            metadata['hashes']['metadata.json'] = '0' * 64
            (root / 'metadata.json').write_text(json.dumps(metadata))
            with self.assertRaisesRegex(ValueError, 'metadata.json'):
                geometry.export(root, root / 'out',
                                geometry.parse_terrain('land=plains,sea=ocean,lake=inland_water'))


if __name__ == '__main__':
    unittest.main()
