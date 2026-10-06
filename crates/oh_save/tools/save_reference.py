"""Independent integer-only postcard/FNV and ledger reference for the M1 fixture.
No product serializer/hash/formula is imported or invoked by this module.
"""
def varint(n):
    assert 0 <= n < 1 << 64
    out = bytearray()
    while n >= 128:
        out.append((n & 127) | 128)
        n >>= 7
    out.append(n)
    return bytes(out)

def signed(n):
    assert -(1 << 63) <= n < 1 << 63
    return varint((n << 1) ^ (n >> 63))

def string(s):
    encoded = s.encode('utf-8')
    return varint(len(encoded)) + encoded

def optional(value, encode=varint):
    return b'\x00' if value is None else b'\x01' + encode(value)

def sequence(values, encode):
    return varint(len(values)) + b''.join(encode(v) for v in values)

def mapping(values):
    assert all(a[0] < b[0] for a, b in zip(values, values[1:]))
    return sequence(values, lambda v: string(v[0]) + signed(v[1]))

def date(value):
    return varint(value['year']) + bytes([value['month'], value['day']])

def modifier(value):
    return (string(value['source']) + string(value['target_stat']) +
            varint(('Add', 'Mul').index(value['op'])) + signed(value['value']) +
            optional(value['expires']))

def entry(value):
    return (optional(value['source'], string) + varint(('Base', 'Add', 'Mul').index(value['op'])) +
            signed(value['value']) + signed(value['accumulated']))

def ledger(value):
    return (string(value['target_stat']) + varint(value['tick']) + signed(value['value']) +
            sequence(value['entries'], entry))

def nation(v):
    return varint(v['id']) + string(v['tag']) + string(v['government']) + mapping(v['support'])

def state_world(v):
    return (varint(v['id']) + varint(v['owner']) + signed(v['population']) +
            mapping(v['resources']) + mapping(v['buildings']) + signed(v['base']) +
            sequence(v['modifiers'], modifier) + signed(v['infrastructure']) + ledger(v['ledger']))

def province(v):
    return varint(v['id']) + optional(v['state']) + optional(v['owner']) + optional(v['controller'])

def queue(v):
    name, argument = next(iter(v['command'].items()))
    opcode = ('Pause', 'SetSpeed').index(name)
    return varint(v['tick']) + varint(v['nation']) + varint(v['sequence']) + varint(opcode) + bytes([int(argument)])

def canonical(dto):
    s, c = dto['state'], dto['config']
    data = (string(s['scenario']) + varint(s['seed']) + varint(s['tick']) + date(s['date']) +
            bytes([s['hour'], int(s['paused']), s['speed']]) +
            b''.join(varint(v) for v in c['speed_ms_per_tick']) + bytes([c['initial_speed']]) +
            sequence(dto['queue'], queue))
    w = dto['world']
    if w is not None:
        i = w['inputs']
        data += (b'\x01' + varint(w['definitions_hash']) + sequence(i['nations'], nation) +
                 sequence(i['states'], state_world) + sequence(i['provinces'], province))
    return data

def fnv(data):
    h = 0xcbf29ce484222325
    for b in data:
        h = ((h ^ b) * 0x100000001b3) & ((1 << 64) - 1)
    return f'{h:016x}'

def pack_hash(root):
    # Whole relative-path sort, exact bytes, postcard Vec<(String,Vec<u8>)>.
    files = sorted((p.relative_to(root).as_posix(), p.read_bytes()) for p in root.rglob('*') if p.is_file())
    encoded = varint(len(files)) + b''.join(string(name) + varint(len(data)) + data for name, data in files)
    return int(fnv(encoded), 16)

def verify_ledger(dto):
    tick = dto['state']['tick']
    for s in dto['world']['inputs']['states']:
        value = s['base']
        rows = [{'source': None, 'op': 'Base', 'value': value, 'accumulated': value}]
        active = sorted((m for m in s['modifiers'] if m['expires'] is None or tick < m['expires']),
                        key=lambda m: (('Add', 'Mul').index(m['op']), m['source']))
        for m in active:
            value = value + m['value'] if m['op'] == 'Add' else (value * m['value']) >> 32
            assert -(1 << 63) <= value < 1 << 63
            rows.append({'source': m['source'], 'op': m['op'], 'value': m['value'], 'accumulated': value})
        assert s['ledger'] == {'target_stat': 'infrastructure', 'tick': tick, 'value': value, 'entries': rows}
        assert s['infrastructure'] == value

def verify_capture(report):
    assert report['fixture'] == 'actual-m1-expiry-v1'
    split, end = report['split'], report['continuous']
    assert split['state']['tick'] == 48 and end['state']['tick'] == 96
    assert split['state']['date'] == {'year': 2000, 'month': 3, 'day': 1}
    assert end['state']['date'] == {'year': 2000, 'month': 3, 'day': 3}
    assert split['state']['seed'] == end['state']['seed'] == 7
    assert len(split['queue']) == 5 and end['queue'] == []
    assert split['world']['inputs']['states'][0]['infrastructure'] == 12884901890
    assert end['world']['inputs']['states'][0]['infrastructure'] == 12884901888
    assert len(end['world']['inputs']['states'][0]['modifiers']) == 3
    assert len(split['world']['inputs']['nations']) == 2
    assert len(split['world']['inputs']['provinces']) == 6
    verify_ledger(split)
    verify_ledger(end)
    encoded = canonical(split)
    assert encoded.hex() == report['canonical_hex']
    assert fnv(encoded) == report['split_hash']
    assert fnv(canonical(end)) == report['continuous_hash']
