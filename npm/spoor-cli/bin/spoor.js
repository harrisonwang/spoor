#!/usr/bin/env node
const { spawnSync } = require('child_process');

const pkg = `@harrisonwang/spoor-cli-${process.platform}-${process.arch}`;
const exe = process.platform === 'win32' ? 'spoor.exe' : 'spoor';

let binary;
try {
  binary = require.resolve(`${pkg}/bin/${exe}`);
} catch {
  console.error(
    `spoor: no prebuilt binary for ${process.platform}-${process.arch}.\n` +
    `Expected optional dependency "${pkg}" to be installed.\n` +
    `If your platform is supported, try: npm install --include=optional @harrisonwang/spoor-cli\n` +
    `Or build from source: https://github.com/harrisonwang/spoor`
  );
  process.exit(1);
}

const result = spawnSync(binary, process.argv.slice(2), { stdio: 'inherit' });
if (result.error) {
  console.error(`spoor: failed to launch ${binary}: ${result.error.message}`);
  process.exit(1);
}
process.exit(result.status ?? 1);
