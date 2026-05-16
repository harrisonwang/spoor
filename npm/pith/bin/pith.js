#!/usr/bin/env node
const { spawnSync } = require('child_process');

const pkg = `@harrisonwang/pith-${process.platform}-${process.arch}`;
const exe = process.platform === 'win32' ? 'pith.exe' : 'pith';

let binary;
try {
  binary = require.resolve(`${pkg}/bin/${exe}`);
} catch {
  console.error(
    `pith: no prebuilt binary for ${process.platform}-${process.arch}.\n` +
    `Expected optional dependency "${pkg}" to be installed.\n` +
    `If your platform is supported, try: npm install --include=optional @harrisonwang/pith\n` +
    `Or build from source: https://github.com/harrisonwang/pith`
  );
  process.exit(1);
}

const result = spawnSync(binary, process.argv.slice(2), { stdio: 'inherit' });
if (result.error) {
  console.error(`pith: failed to launch ${binary}: ${result.error.message}`);
  process.exit(1);
}
process.exit(result.status ?? 1);
