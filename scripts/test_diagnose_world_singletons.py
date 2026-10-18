"""Tiny synthetic tests only. No real-world calculation in this suite."""
import array
from collections import Counter
import json
from pathlib import Path
import random
import tempfile
import unittest

import diagnose_world_singletons as diagnostic
from export_pack_geometry import absorb


class DiagnosticTests(unittest.TestCase):
    def check(self, pixels, width, kinds):
        original = list(pixels)
        report = diagnostic.analyse(pixels, width, len(pixels) // width, kinds)
        self.assertEqual(list(pixels), original)
        self.assertEqual(report, diagnostic.analyse(pixels, width, len(pixels) // width, kinds, reverse=True))
        self.assertTrue(report['original_2_to_8_pixels_unchanged_first'])
        self.assertTrue(report['original_2_to_8_pixels_unchanged_second'])
        return report

    def test_candidates_same_kind_border_and_id_tie(self):
        kinds = {1: 'land', 2: 'sea', 3: 'land'}
        pixels = [2, 2, 3, 3, 2, 1, 3, 3]
        report = self.check(pixels, 4, kinds)
        self.assertEqual(report['first_mapping'], {'1': 3})
        self.assertEqual(report['blocked_count'], 0)
        normalized, merges = absorb(array.array('H', pixels), 4, 2, kinds)
        self.assertEqual(report['first_mapping'], {str(m['id']): m['target'] for m in merges})
        self.assertEqual(normalized[5], 3)
        kinds[3] = 'sea'
        report = self.check([2, 2, 3, 3, 2, 1, 3, 3, 2, 3, 3, 3], 4, kinds)
        self.assertEqual(report['first_mapping'], {'1': 2})
        report = self.check([2, 2, 3, 3, 2, 1, 2, 3], 4, kinds)
        self.assertEqual(report['first_mapping'], {'1': 2})
        self.assertEqual(report['first_changed_kind_pixels'], 1)

    def test_no_candidate_and_chain_blocked_with_explicit_hypothesis(self):
        pixels = [3, 3, 4, 3, 3, 3, 2, 1, 5, 3, 3, 3, 6, 3, 3]
        kinds = {i: 'sea' for i in range(1, 7)}
        report = self.check(pixels, 5, kinds)
        self.assertEqual(report['blocked_ids'], [1])
        self.assertNotIn('1', report['first_mapping'])
        self.assertEqual(report['hypothetical_second_mapping'], {'1': 3})
        self.assertEqual(report['hypothetical_remaining_count'], 0)
        row = next(r for r in report['singletons'] if r['id'] == 1)
        self.assertEqual(row['neighbours']['W']['id'], 2)
        self.assertEqual(row['neighbours_after_first']['W']['id'], 3)
        with self.assertRaisesRegex(ValueError, 'no original'):
            absorb(array.array('H', pixels), 5, 3, kinds)
        all_single = self.check([1, 2, 3, 4], 2, {i: 'land' for i in range(1, 5)})
        self.assertEqual(all_single['blocked_count'], 4)
        self.assertEqual(all_single['hypothetical_remaining_ids'], [1, 2, 3, 4])

    def test_horizontal_wrap_and_no_vertical_wrap(self):
        kinds = {1: 'land', 2: 'land', 3: 'sea'}
        horizontal = self.check([1, 3, 3, 2, 3, 3, 3, 2], 4, kinds)
        self.assertEqual(horizontal['first_mapping'], {'1': 2})
        row = horizontal['singletons'][0]
        self.assertEqual(row['neighbours']['W']['id'], 2)
        self.assertIsNone(row['neighbours']['N'])
        vertical = self.check([3, 1, 3, 3, 3, 3, 2, 2, 2], 3, kinds)
        self.assertEqual(vertical['first_mapping'], {'1': 3})
        self.assertIsNone(vertical['singletons'][0]['neighbours']['N'])

    def test_independent_small_snapshot_oracle_and_order(self):
        rng = random.Random(92961)
        for _ in range(100):
            width, height = 5, 4
            pixels = [rng.randint(1, 9) for _ in range(width * height)]
            kinds = {p: rng.choice(['land', 'sea', 'lake']) for p in set(pixels)}
            report = self.check(pixels, width, kinds)
            snapshot = list(pixels)
            for key in ['first_mapping', 'hypothetical_second_mapping']:
                areas = Counter(snapshot)
                expected = {}
                for pos, province in enumerate(snapshot):
                    if areas[province] != 1:
                        continue
                    x, y = pos % width, pos // width
                    coords = [((x - 1) % width, y), ((x + 1) % width, y)]
                    coords += [(x, yy) for yy in [y - 1, y + 1] if 0 <= yy < height]
                    eligible = [snapshot[yy * width + xx] for xx, yy in coords
                                if areas[snapshot[yy * width + xx]] > 1]
                    if eligible:
                        expected[str(province)] = sorted(set(eligible), key=lambda p: (
                            kinds[p] != kinds[province], -eligible.count(p), p))[0]
                self.assertEqual(report[key], expected)
                snapshot = [expected.get(str(p), p) for p in snapshot]
            self.assertEqual(sum(n == 1 for n in Counter(snapshot).values()),
                             report['hypothetical_remaining_count'])

    def test_input_receipts_zero_based_identity_tampering_and_no_writes(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root / 'input'
            source.mkdir()
            pixels = [1, 1, 2, 2, 3, 3, 3, 3]
            (source / 'index.bin').write_bytes(b''.join((p - 1).to_bytes(2, 'little') for p in pixels))
            (source / 'provinces.json').write_text(json.dumps([
                dict(id=p, kind='land', pixels=pixels.count(p)) for p in [1, 2, 3]]))
            (source / 'adjacency.json').write_text('[]')
            hashes = {name: diagnostic.digest((source / name).read_bytes())
                      for name in ['index.bin', 'provinces.json', 'adjacency.json']}
            (source / 'metadata.json').write_text(json.dumps(dict(
                schema='world-preview-v1', byte_order='u16-le', extent=[-180, -90, 180, 90],
                width=4, height=2, hashes=hashes)))
            before = {p.name: p.read_bytes() for p in source.iterdir()}
            loaded, _, _, kinds, receipts = diagnostic.load_input(source)
            self.assertEqual(list(loaded), pixels)
            self.assertEqual(set(kinds), {1, 2, 3})
            self.assertEqual(receipts['index.bin']['bytes'], 16)
            a = diagnostic.diagnose(source, root / 'a', 'a' * 40)
            b = diagnostic.diagnose(source, root / 'b', 'a' * 40)
            self.assertEqual(a, b)
            self.assertFalse(a['productComplete'])
            self.assertEqual((root / 'a/diagnostic.json').read_bytes(), (root / 'b/diagnostic.json').read_bytes())
            self.assertEqual(before, {p.name: p.read_bytes() for p in source.iterdir()})
            for name in hashes:
                (source / name).write_bytes(before[name] + b' ')
                with self.assertRaisesRegex(ValueError, 'sha256 mismatch'):
                    diagnostic.diagnose(source, root / 'bad', 'a' * 40)
                self.assertFalse((root / 'bad').exists())
                (source / name).write_bytes(before[name])


if __name__ == '__main__':
    unittest.main()
