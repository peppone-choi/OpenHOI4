"""Integer policy and actual native evidence comparison, never skip a sample."""
def regression(baseline, current, threshold):
    if len(baseline) != 2 or len(current) != 2 or type(threshold) is not int or threshold != 15:
        raise ValueError('require exactly two pairs and the approved 15 percent threshold')
    for value in baseline + current:
        if type(value) is not int or value <= 0:
            raise ValueError('measurement must be a positive integer native nanosecond count')
    return all(c * 100 > b * (100 + threshold) for b,c in zip(baseline,current))

def validate_sample(sample, policy):
    for key in ('elapsed_ns','steps','seed','tick'):
        if type(sample.get(key)) is not int or sample[key] <= 0:
            raise ValueError('missing/invalid native ' + key)
    if sample['steps'] != policy['steps'] or sample['tick'] != policy['steps'] or sample['seed'] != policy['seed'] or sample.get('scenario') != policy['scenario'] or sample.get('ended') is not False:
        raise ValueError('incomplete/different workload')
    for key in ('dto','canonical_hex','hash'):
        if not sample.get(key): raise ValueError('missing full state ' + key)
    h = sample['hash']
    if type(h) is not str or len(h) != 16 or any(c not in '0123456789abcdef' for c in h):
        raise ValueError('invalid hash')
    for key in ('pack_hash','pack_hash_after'):
        value=sample.get(key)
        if type(value) is not str or len(value)!=16 or any(c not in '0123456789abcdef' for c in value): raise ValueError('missing/invalid actual '+key)
    if sample['pack_hash']!=sample['pack_hash_after']: raise ValueError('native pack changed during run')
    canonical = bytes.fromhex(sample['canonical_hex'])
    value = 0xcbf29ce484222325
    for byte in canonical: value = ((value ^ byte) * 0x100000001b3) & ((1 << 64)-1)
    if f'{value:016x}' != h: raise ValueError('canonical/hash disagreement')

def compare_samples(baseline, current, policy):
    if len(baseline) != 2 or len(current) != 2: raise ValueError('missing pair')
    for sample in baseline + current: validate_sample(sample,policy)
    expected={k:baseline[0][k] for k in ('dto','canonical_hex','hash','pack_hash','pack_hash_after','steps','seed','scenario','tick','ended')}
    for sample in baseline + current:
        if {k:sample[k] for k in expected} != expected: raise ValueError('native completed full state mismatch')
    return regression([s['elapsed_ns'] for s in baseline],[s['elapsed_ns'] for s in current],policy['threshold_percent'])
