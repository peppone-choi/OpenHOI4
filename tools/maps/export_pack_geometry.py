#!/usr/bin/env python3
"""Deterministic, non-product world-preview geometry exporter (stdlib only)."""
import argparse
import array
from collections import Counter

import hashlib
import json
from pathlib import Path
import re
import struct
import sys
import zlib

# ADR-0702 §1 exact published TOML lexeme, never a platform float calculation.
KM_PER_PIXEL = '4.8865025493'
KINDS = {'land', 'sea', 'lake'}
MARKER = 'geometry harness, not product pack'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def json_bytes(value):
    return (json.dumps(value, sort_keys=True, separators=(',', ':')) + '\n').encode()


def parse_terrain(text):
    try:
        pairs = [entry.split('=') for entry in text.split(',')]
        result = dict(pairs)
    except ValueError as exc:
        raise ValueError('terrain-map requires land=ID,sea=ID,lake=ID') from exc
    if (len(pairs) != 3 or set(result) != KINDS or
            any(not re.fullmatch(r'[a-z0-9_]+', v) for v in result.values())):
        raise ValueError('terrain-map requires unique land/sea/lake and valid terrain IDs')
    return result


def rgb(province):
    return (province >> 8, province & 255, 0)


def neighbours(position, width, height):
    y, x = divmod(position, width)
    yield y * width + (x - 1) % width
    yield y * width + (x + 1) % width
    if y:
        yield position - width
    if y + 1 < height:
        yield position + width


def absorb(index, width, height, kinds):
    """Freeze original areas; decide every singleton before writing any pixels."""
    if len(index) != width * height or width < 2 or height < 2:
        raise ValueError('index dimensions must match and be at least 2x2')
    counts = Counter(index)
    if set(counts) != set(kinds):
        raise ValueError('province IDs and index IDs differ')
    positions = {province: i for i, province in enumerate(index) if counts[province] == 1}
    merges = []
    for province, position in sorted(positions.items()):
        borders = Counter(index[n] for n in neighbours(position, width, height)
                          if counts[index[n]] >= 2)
        if not borders:
            raise ValueError(f'singleton {province}: no original area>=2 neighbour')
        same = [p for p in borders if kinds[p] == kinds[province]]
        target = min(same or borders, key=lambda p: (-borders[p], p))
        y, x = divmod(position, width)
        merges.append(dict(id=province, target=target, old_kind=kinds[province],
                           new_kind=kinds[target], pixel=[x, y]))
    result = array.array('H', index)
    for merge in merges:
        x, y = merge['pixel']
        result[y * width + x] = merge['target']
    return result, merges


def edges(index, width, height):
    result = set()
    for y in range(height):
        row = index[y * width:(y + 1) * width]
        pairs = zip(row, row[1:])
        for a, b in pairs:
            if a != b:
                result.add((min(a, b), max(a, b)))
        if row[0] != row[-1]:
            result.add((min(row[0], row[-1]), max(row[0], row[-1])))
        if y:
            for a, b in zip(row, index[(y - 1) * width:y * width]):
                if a != b:
                    result.add((min(a, b), max(a, b)))
    return result


def coastal_from_edges(pairs, kinds):
    coastal = set()
    for a, b in pairs:
        if kinds[a] == 'land' and kinds[b] == 'sea':
            coastal.add(a)
        if kinds[b] == 'land' and kinds[a] == 'sea':
            coastal.add(b)
    return coastal


def coastal_ids(index, width, height, kinds):
    return coastal_from_edges(edges(index, width, height), kinds)


def png_bytes(index, width, height):
    def chunk(kind, data):
        return (struct.pack('>I', len(data)) + kind + data +
                struct.pack('>I', zlib.crc32(kind + data)))
    compressor = zlib.compressobj(9)
    compressed = []
    for y in range(height):
        row = b'\0' + b''.join(bytes(rgb(p)) for p in index[y * width:(y + 1) * width])
        compressed.append(compressor.compress(row))
    compressed.append(compressor.flush())
    return (b'\x89PNG\r\n\x1a\n' +
            chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0)) +
            chunk(b'IDAT', b''.join(compressed)) + chunk(b'IEND', b''))


def export(source, out, terrain):
    source, out = Path(source), Path(out)
    if set(terrain) != KINDS or any(not re.fullmatch(r'[a-z0-9_]+', v)
                                  for v in terrain.values()):
        raise ValueError('explicit land/sea/lake terrain mapping required')
    if out.exists():
        raise ValueError('output must be a new directory')
    metadata_raw = (source / 'metadata.json').read_bytes()
    metadata = json.loads(metadata_raw)
    declared = metadata.get('hashes', {})
    required = {'index.bin', 'provinces.json', 'adjacency.json'}
    if set(declared) != required:
        raise ValueError('metadata.json hashes must cover exactly index.bin, provinces.json, adjacency.json; no self-reference')
    inputs = {}
    for name in sorted(required):
        raw = (source / name).read_bytes()
        if digest(raw) != declared[name]:
            raise ValueError(f'{name}: sha256 mismatch')
        inputs[name] = raw
    width, height = metadata['width'], metadata['height']
    if (metadata.get('schema') != 'world-preview-v1' or
            metadata.get('byte_order') != 'u16-le' or
            metadata.get('extent') != [-180, -90, 180, 90] or
            type(width) is not int or type(height) is not int or
            width < 2 or height < 2 or len(inputs['index.bin']) != width * height * 2):
        raise ValueError('metadata.json: unsupported geometry contract')
    index = array.array('H')
    index.frombytes(inputs['index.bin'])
    if sys.byteorder != 'little':
        index.byteswap()
    # world-preview-v1 stores dense zero-based indices; provinces.json IDs are
    # one-based. Restore identity, never compact or renumber after absorption.
    index = array.array('H', (p + 1 for p in index))
    provinces = json.loads(inputs['provinces.json'])
    kinds = {}
    original_counts = Counter(index)
    for p in provinces:
        province = p['id']
        if (type(province) is not int or not 1 <= province <= 65535 or
                province in kinds or p['kind'] not in KINDS or
                p['pixels'] != original_counts[province]):
            raise ValueError(f'provinces.json: invalid province {province}')
        kinds[province] = p['kind']
    normalized, merges = absorb(index, width, height, kinds)
    counts = Counter(normalized)
    pairs = edges(normalized, width, height)
    coastal = coastal_from_edges(pairs, kinds)
    rows = ['id,r,g,b,kind,terrain,coastal,island\n']
    for province in sorted(counts):
        r, g, b = rgb(province)
        kind = kinds[province]
        rows.append(f'{province},{r},{g},{b},{kind},{terrain[kind]},{str(province in coastal).lower()},false\n')
    files = {
        'provinces.png': png_bytes(normalized, width, height),
        'provinces.csv': ''.join(rows).encode(),
        'adjacency_overrides.csv': b'a,b,kind\n',
        'regions.toml': (f'km_per_pixel = {KM_PER_PIXEL}\nprojection = "equirectangular"\nextent = [-180, -90, 180, 90]\n').encode(),
    }
    seam = {tuple(sorted((normalized[y * width], normalized[(y + 1) * width - 1])))
            for y in range(height)
            if normalized[y * width] != normalized[(y + 1) * width - 1]}
    report = dict(marker=MARKER, width=width, height=height,
                  input_sha256={**declared, 'metadata.json': digest(metadata_raw)},
                  hash_scope='metadata.json is recorded, not authenticated; schema declares only the other three input hashes. report.json is excluded from its own output hashes.',
                  output_sha256={name: digest(raw) for name, raw in files.items()},
                  terrain_map=terrain, km_per_pixel=KM_PER_PIXEL,
                  original_count=len(kinds), province_count=len(counts),
                  original_kind_pixels={k: sum(n for p, n in original_counts.items() if kinds[p] == k) for k in sorted(KINDS)},
                  kind_pixels={k: sum(n for p, n in counts.items() if kinds[p] == k) for k in sorted(KINDS)},
                  merged_count=len(merges), merges=merges,
                  changed_kind_pixels=sum(m['old_kind'] != m['new_kind'] for m in merges),
                  edge_count=len(pairs), seam_samples=sorted(seam)[:3],
                  png_bytes=len(files['provinces.png']))
    files['report.json'] = json_bytes(report)
    out.mkdir(parents=True)
    for name, raw in files.items():
        (out / name).write_bytes(raw)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', required=True, type=Path)
    parser.add_argument('--out', required=True, type=Path)
    parser.add_argument('--terrain-map', required=True)
    args = parser.parse_args()
    try:
        report = export(args.input, args.out, parse_terrain(args.terrain_map))
    except (ValueError, OSError, KeyError, TypeError) as exc:
        parser.exit(1, f'{exc}\n')
    print(f'{MARKER}: {report["province_count"]} provinces, {report["merged_count"]} absorbed')


if __name__ == '__main__':
    main()
