#!/usr/bin/env python3
"""Read-only D1 diagnosis. Hypothetical rounds are NOT exporter policy or product PASS."""
import argparse
import array
from collections import Counter
import json
from pathlib import Path
import subprocess
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools/maps'))
from export_pack_geometry import KINDS, digest, json_bytes, neighbours  # noqa: E402

MARKER = 'diagnostic only; hypothetical round two is NOT approved D1 or product PASS'


def analyse(index, width, height, kinds, reverse=False):
    """Snapshot decisions; never mutate input, chain within a round, or search remotely."""
    if width < 2 or height < 2 or len(index) != width * height:
        raise ValueError('index dimensions must match and be at least 2x2')
    counts = Counter(index)
    if set(counts) != set(kinds) or not set(kinds.values()) <= KINDS:
        raise ValueError('index IDs/kinds differ')
    positions = {p: i for i, p in enumerate(index) if counts[p] == 1}

    def adjacent(position, snapshot):
        y, x = divmod(position, width)
        slots = [('W', y * width + (x - 1) % width),
                 ('E', y * width + (x + 1) % width),
                 ('N', position - width if y else None),
                 ('S', position + width if y + 1 < height else None)]
        return {direction: None if pos is None else dict(
            id=snapshot[pos], original_area=counts[snapshot[pos]],
            kind=kinds[snapshot[pos]], pixel=[pos % width, pos // width])
                for direction, pos in slots}

    def decisions(snapshot, areas, ids):
        mapping = {}
        for p in sorted(ids, reverse=reverse):
            borders = Counter(snapshot[n] for n in neighbours(positions[p], width, height)
                              if areas[snapshot[n]] >= 2)
            same = [target for target in borders if kinds[target] == kinds[p]]
            if borders:
                mapping[p] = min(same or borders, key=lambda t: (-borders[t], t))
        return mapping

    first = decisions(index, counts, positions)
    blocked = sorted(set(positions) - set(first))
    after = array.array('H', index)
    for p, target in first.items():
        after[positions[p]] = target
    after_counts = counts.copy()
    for p, target in first.items():
        after_counts[target] += 1
        del after_counts[p]
    # Only this explicitly hypothetical round uses the post-round-one snapshot.
    second = decisions(after, after_counts, blocked)
    final = array.array('H', after)
    for p, target in second.items():
        final[positions[p]] = target

    def kind_counts(ids):
        return {kind: sum(kinds[p] == kind for p in ids) for kind in sorted(KINDS)}

    def changes(mapping):
        return sum(kinds[p] != kinds[t] for p, t in mapping.items())

    # Only original singletons may be written, so every original 2..8px pixel
    # remains its original ID. Count-based growth by absorption is separate.
    small_positions = [i for i, p in enumerate(index) if 2 <= counts[p] <= 8]
    unchanged_first = all(after[i] == index[i] for i in small_positions)
    unchanged_second = all(final[i] == index[i] for i in small_positions)
    if not unchanged_first or not unchanged_second:
        raise ValueError('original 2..8px pixels changed')
    rows = []
    for p in sorted(positions):
        pos = positions[p]
        rows.append(dict(id=p, kind=kinds[p], pixel=[pos % width, pos // width],
                         neighbours=adjacent(pos, index), first_target=first.get(p),
                         neighbours_after_first=adjacent(pos, after) if p in blocked else None,
                         hypothetical_second_target=second.get(p)))
    remaining = sorted(set(blocked) - set(second))
    return dict(marker=MARKER, width=width, height=height,
                singleton_count=len(positions), singleton_by_kind=kind_counts(positions),
                blocked_count=len(blocked), blocked_by_kind=kind_counts(blocked), blocked_ids=blocked,
                first_mapping={str(p): first[p] for p in sorted(first)},
                first_merged_count=len(first), first_changed_kind_pixels=changes(first),
                hypothetical_second_mapping={str(p): second[p] for p in sorted(second)},
                hypothetical_second_merged_count=len(second),
                hypothetical_second_changed_kind_pixels=changes(second),
                hypothetical_total_changed_kind_pixels=changes(first) + changes(second),
                hypothetical_remaining_count=len(remaining),
                hypothetical_remaining_by_kind=kind_counts(remaining), hypothetical_remaining_ids=remaining,
                original_2_to_8_provinces=sum(2 <= n <= 8 for n in counts.values()),
                original_2_to_8_pixels=len(small_positions),
                original_2_to_8_pixels_unchanged_first=unchanged_first,
                original_2_to_8_pixels_unchanged_second=unchanged_second,
                small_area_meaning='Original pixels/IDs unchanged; target areas may grow by absorption.',
                singletons=rows)


def load_input(source):
    metadata_raw = (source / 'metadata.json').read_bytes()
    metadata = json.loads(metadata_raw)
    required = {'index.bin', 'provinces.json', 'adjacency.json'}
    if set(metadata.get('hashes', {})) != required:
        raise ValueError('metadata hashes must cover exactly the three inputs')
    raw = {}
    receipts = {'metadata.json': dict(sha256=digest(metadata_raw), bytes=len(metadata_raw))}
    for name in sorted(required):
        raw[name] = (source / name).read_bytes()
        actual = digest(raw[name])
        if actual != metadata['hashes'][name]:
            raise ValueError(f'{name}: sha256 mismatch')
        receipts[name] = dict(sha256=actual, bytes=len(raw[name]))
    width, height = metadata['width'], metadata['height']
    if (metadata.get('schema') != 'world-preview-v1' or
            metadata.get('byte_order') != 'u16-le' or
            metadata.get('extent') != [-180, -90, 180, 90] or
            type(width) is not int or type(height) is not int or
            width < 2 or height < 2 or len(raw['index.bin']) != width * height * 2):
        raise ValueError('unsupported geometry contract')
    index = array.array('H')
    index.frombytes(raw['index.bin'])
    if sys.byteorder != 'little':
        index.byteswap()
    index = array.array('H', (p + 1 for p in index))
    counts = Counter(index)
    kinds = {}
    for p in json.loads(raw['provinces.json']):
        ident = p['id']
        if (type(ident) is not int or not 1 <= ident <= 65535 or ident in kinds or
                p['kind'] not in KINDS or p['pixels'] != counts[ident]):
            raise ValueError(f'invalid province {ident}')
        kinds[ident] = p['kind']
    if set(counts) != set(kinds):
        raise ValueError('index IDs and provinces differ')
    return index, width, height, kinds, receipts


def diagnose(source, out, head):
    if out.exists():
        raise ValueError('output must be a new directory')
    if not (len(head) == 40 and all(c in '0123456789abcdef' for c in head)):
        raise ValueError('head must be exact lowercase SHA')
    index, width, height, kinds, receipts = load_input(source)
    report = analyse(index, width, height, kinds)
    report.update(head_sha=head, input_files=receipts,
                  hash_scope='metadata is recorded, not self-authenticated; other inputs verified against metadata',
                  productComplete=False, exporter_executed=False, policy_changed=False)
    # Source files must remain byte-identical; diagnostic never writes to them.
    for name, receipt in receipts.items():
        if digest((source / name).read_bytes()) != receipt['sha256']:
            raise ValueError(f'{name}: input changed during diagnosis')
    report['source_unchanged'] = True
    keys = ['singleton_count', 'singleton_by_kind', 'blocked_count', 'blocked_by_kind',
            'first_merged_count', 'first_changed_kind_pixels',
            'hypothetical_second_merged_count', 'hypothetical_remaining_count',
            'hypothetical_remaining_by_kind', 'hypothetical_total_changed_kind_pixels',
            'original_2_to_8_provinces', 'original_2_to_8_pixels',
            'original_2_to_8_pixels_unchanged_first', 'original_2_to_8_pixels_unchanged_second']
    summary = '\n'.join([MARKER, f'head_sha: {head}'] +
                        [f'{key}: {json.dumps(report[key], sort_keys=True)}' for key in keys]) + '\n'
    out.mkdir(parents=True)
    (out / 'diagnostic.json').write_bytes(json_bytes(report))
    (out / 'summary.txt').write_text(summary, encoding='utf-8')
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', required=True, type=Path)
    parser.add_argument('--out', required=True, type=Path)
    args = parser.parse_args()
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    try:
        report = diagnose(args.input, args.out, head)
    except (ValueError, OSError, KeyError, TypeError, OverflowError) as exc:
        parser.exit(1, f'{exc}\n')
    print(f'{MARKER}: {report["singleton_count"]} singletons; {report["blocked_count"]} blocked')


if __name__ == '__main__':
    main()
