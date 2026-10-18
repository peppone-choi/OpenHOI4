"""Small, synthetic contracts; real-world load_map runs only on hosted CI."""
import array
import hashlib
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest
import zlib

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
        tied = array.array('H', [2, 2, 3, 3, 2, 1, 3, 3, 2, 3, 3, 3])
        result, _ = geometry.absorb(tied, 4, 3, kinds)
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
        raw = b''.join((i - 1).to_bytes(2, 'little') for i in pixels)
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


def world_check(source, first, second, expected_path):
    """Independent oracle: decode PNG; replay report against ORIGINAL pixels.

    No exporter adjacency/absorb/PNG function is used here. The oracle is stored
    outside the generated map so Rust compares against a separate implementation.
    """
    source, first, second = map(Path, (source, first, second))
    assert {p.name: p.read_bytes() for p in first.iterdir()} == {
        p.name: p.read_bytes() for p in second.iterdir()}
    report = json.loads((first / 'report.json').read_text())
    metadata = json.loads((source / 'metadata.json').read_text())
    w, h = metadata['width'], metadata['height']
    original = array.array('H')
    original.frombytes((source / 'index.bin').read_bytes())
    if sys.byteorder != 'little':
        original.byteswap()
    original = array.array('H', (p + 1 for p in original))
    from collections import Counter
    areas = Counter(original)
    kinds = {p['id']: p['kind'] for p in json.loads((source / 'provinces.json').read_text())}
    remap = {m['id']: m['target'] for m in report['merges']}
    assert set(remap) == {p for p, n in areas.items() if n == 1}
    for m in report['merges']:
        x, y = m['pixel']
        i = y * w + x
        assert original[i] == m['id']
        adjacent = [original[y * w + (x + dx) % w] for dx in (-1, 1)]
        adjacent += [original[ny * w + x] for ny in (y - 1, y + 1) if 0 <= ny < h]
        candidates = Counter(p for p in adjacent if areas[p] >= 2)
        same = {p: n for p, n in candidates.items() if kinds[p] == kinds[m['id']]}
        best = sorted((same or candidates), key=lambda p: (-(same or candidates)[p], p))[0]
        assert best == m['target'] and best not in remap
        assert m['old_kind'] == kinds[m['id']] and m['new_kind'] == kinds[best]
    # Decode the restricted RGB PNG independently, including chunk CRCs.
    png = (first / 'provinces.png').read_bytes()
    assert png[:8] == b'\x89PNG\r\n\x1a\n'
    offset, compressed = 8, bytearray()
    while offset < len(png):
        size = struct.unpack_from('>I', png, offset)[0]
        tag = png[offset + 4:offset + 8]
        data = png[offset + 8:offset + 8 + size]
        assert zlib.crc32(tag + data) == struct.unpack_from('>I', png, offset + 8 + size)[0]
        if tag == b'IHDR':
            assert struct.unpack('>IIBBBBB', data) == (w, h, 8, 2, 0, 0, 0)
        if tag == b'IDAT':
            compressed.extend(data)
        offset += size + 12
    raw = zlib.decompress(compressed)
    assert len(raw) == h * (1 + w * 3)
    normalized = array.array('H')
    for y in range(h):
        start = y * (1 + w * 3)
        assert raw[start] == 0
        row = raw[start + 1:start + 1 + w * 3]
        assert not any(row[2::3])
        normalized.extend((r << 8) | g for r, g in zip(row[0::3], row[1::3]))
    for i, p in enumerate(original):
        assert normalized[i] == remap.get(p, p), (i, p)
    # Deliberately different traversal from exporter: columns, then rows.
    pairs = set()
    def add(a, b):
        if a != b:
            pairs.add(tuple(sorted((a, b))))
    for x in range(w):
        column = normalized[x::w]
        for a, b in zip(column, column[1:]):
            add(a, b)
        right = normalized[(x + 1) % w::w]
        for a, b in zip(column, right):
            add(a, b)
    before = {tuple(pair) for pair in json.loads((source / 'adjacency.json').read_text())}
    mapped = {tuple(sorted((remap.get(a, a), remap.get(b, b)))) for a, b in before
              if remap.get(a, a) != remap.get(b, b)}
    assert pairs == mapped  # no non-local edges created by singleton absorption
    counts = Counter(normalized)
    assert len(counts) == 24889 - len(remap) == report['province_count']
    kind_pixels = {k: sum(n for p, n in counts.items() if kinds[p] == k)
                   for k in ('land', 'sea', 'lake')}
    assert kind_pixels == report['kind_pixels']
    assert report['changed_kind_pixels'] == sum(kinds[p] != kinds[t] for p, t in remap.items())
    assert report['edge_count'] == len(pairs)
    for name, sha in report['output_sha256'].items():
        assert hashlib.sha256((first / name).read_bytes()).hexdigest() == sha
    seam = sorted({tuple(sorted((normalized[y * w], normalized[(y + 1) * w - 1])))
                   for y in range(h) if normalized[y * w] != normalized[(y + 1) * w - 1]})
    assert len(seam) >= 3 and seam[:3] == [tuple(p) for p in report['seam_samples']]
    for cap in (6788, 18014):
        assert cap not in remap
        assert all(cap in normalized[x::w] for x in range(w))
    expected = dict(width=w, height=h, provinces=sorted(counts),
                    counts={str(p): n for p, n in counts.items()},
                    kinds={str(p): kinds[p] for p in counts}, kind_pixels=kind_pixels,
                    edges=sorted(pairs), seam_samples=seam[:3], polar=[6788, 18014])
    Path(expected_path).write_text(json.dumps(expected, sort_keys=True) + '\n')
    print('independent PNG/pixel/kind/edge/determinism oracle PASS', len(counts), len(pairs))


if __name__ == '__main__':
    if len(sys.argv) > 1 and sys.argv[1] == '--world-check':
        world_check(*sys.argv[2:])
    else:
        unittest.main()
