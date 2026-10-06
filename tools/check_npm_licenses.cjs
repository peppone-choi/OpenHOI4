// Check npm's complete lockfile, including dev and other-platform dependencies.
const { resolve } = require('node:path');
const { readFileSync } = require('node:fs');
const allowed = new Set(['MIT', 'Apache-2.0', 'BSD-2-Clause', 'BSD-3-Clause', 'ISC', 'Zlib', 'MPL-2.0', 'Unicode-3.0', 'BSL-1.0', 'CC0-1.0']);

// Closed SPDX subset: exact IDs, AND, OR, and parentheses. Exceptions and
// later-version (+) expressions are outside the project's explicit allow list.
function permitted(source) {
  if (typeof source !== 'string' || !source.trim()) return false;
  const tokens = source.match(/[A-Za-z0-9.-]+|[()]|\S/g);
  let index = 0;
  function atom() {
    const token = tokens[index++];
    if (token === '(') {
      const value = or();
      if (tokens[index++] !== ')') throw new Error('unclosed expression');
      return value;
    }
    if (!token || !/^[A-Za-z0-9][A-Za-z0-9.-]*$/.test(token) || ['AND', 'OR', 'WITH'].includes(token)) {
      throw new Error('invalid license identifier');
    }
    return allowed.has(token);
  }
  function and() {
    let value = atom();
    while (tokens[index] === 'AND') {
      index++;
      const next = atom();
      value = value && next;
    }
    return value;
  }
  function or() {
    let value = and();
    while (tokens[index] === 'OR') {
      index++;
      const next = and();
      value = value || next;
    }
    return value;
  }
  try {
    const result = or();
    return index === tokens.length && result;
  } catch { return false; }
}

function main() {
  const args = process.argv.slice(2);
  if (args.length && (args.length !== 2 || args[0] !== '--root')) {
    throw new Error('Usage: node tools/check_npm_licenses.cjs [--root directory]');
  }
  const directory = args.length ? resolve(args[1]) : resolve(__dirname, '../client');
  const lock = JSON.parse(readFileSync(resolve(directory, 'package-lock.json'), 'utf8'));
  if (lock.lockfileVersion !== 3 || !lock.packages || typeof lock.packages !== 'object') {
    throw new Error('expected npm lockfile version 3 with package inventory');
  }
  let failures = 0;
  let count = 0;
  for (const [path, info] of Object.entries(lock.packages).sort()) {
    if (path === '') continue; // First-party license is pending D-10.
    count++;
    if (!permitted(info.license)) {
      failures++;
      console.error(`[LICENSE] ${path}@${info.version}: unapproved license ${JSON.stringify(info.license)}`);
    }
  }
  if (!count) {
    console.error('[LICENSE] empty dependency inventory');
    failures++;
  }
  console.log(`npm licenses: ${count} dependencies; ${failures} error(s)`);
  return failures ? 1 : 0;
}

try {
  process.exitCode = main();
} catch (error) {
  console.error(`license check failed: ${error.message}`);
  process.exitCode = 1;
}
