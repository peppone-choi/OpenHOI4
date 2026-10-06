# Independent byte-format reference: integer-only ChaCha8, postcard varints, FNV-1a.
from pathlib import Path
MASK = (1 << 32) - 1

def rotate(x, n):
    return ((x << n) | (x >> (32 - n))) & MASK

def quarter(s, a, b, c, d):
    s[a] = (s[a] + s[b]) & MASK; s[d] = rotate(s[d] ^ s[a], 16)
    s[c] = (s[c] + s[d]) & MASK; s[b] = rotate(s[b] ^ s[c], 12)
    s[a] = (s[a] + s[b]) & MASK; s[d] = rotate(s[d] ^ s[a], 8)
    s[c] = (s[c] + s[d]) & MASK; s[b] = rotate(s[b] ^ s[c], 7)

def chacha8(seed):
    initial = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574]
    initial += [int.from_bytes(seed[i:i+4], 'little') for i in range(0, 32, 4)]
    initial += [0, 0, 0, 0]  # 64-bit block counter and 64-bit stream ID.
    s = initial.copy()
    for _ in range(4):
        for indices in [(0,4,8,12), (1,5,9,13), (2,6,10,14), (3,7,11,15), (0,5,10,15), (1,6,11,12), (2,7,8,13), (3,4,9,14)]:
            quarter(s, *indices)
    return [(x+y) & MASK for x, y in zip(s, initial)]

def varint(n):
    result = bytearray()
    while n >= 128:
        result.append((n & 127) | 128)
        n >>= 7
    result.append(n)
    return result

def signed(n):
    return varint((n << 1) ^ (n >> 63))

def fnv(data):
    value = 0xcbf29ce484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
    return value

print('ChaCha8 zero seed:', ', '.join(f'{word:08x}' for word in chacha8(bytes(32))[:8]))
day = 0xfedcba9876543210
ids = [0, 1, 127, (1 << 32)-1]
bits = [-(1 << 63), -1, 1, (1 << 63)-1]
result = varint(day) + varint(len(ids))
for index, entity in enumerate(ids):
    seed = (0x0123456789abcdef).to_bytes(8,'little') + (0x87654321).to_bytes(4,'little') + day.to_bytes(8,'little') + (4).to_bytes(4,'little') + entity.to_bytes(8,'little')
    words = chacha8(seed)
    draws = [words[i] | (words[i+1] << 32) for i in range(0, 8, 2)]
    result += varint(entity) + varint(65535) + varint(128) + varint(0)
    result += signed(bits[index]) + signed(bits[3-index]) + varint(len(draws))
    for draw in draws:
        result += varint(draw)
print(f'core-format-v1: bytes={len(result)}, hash={fnv(result):016x}')
print(f'hex={result.hex()}')
path = Path('crates/oh_core/tests/fixtures')
assert (path / 'core-format-v1.hex').read_text(encoding='utf-8').strip() == result.hex()
assert (path / 'core-format-v1.hash').read_text(encoding='utf-8').strip() == f'{fnv(result):016x}'
print('Existing reference fixtures match; no files were changed.')
