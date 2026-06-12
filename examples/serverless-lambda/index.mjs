import { spawn } from 'node:child_process';
import { mkdtemp, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

export async function handler(event) {
  const dir = await mkdtemp(join(tmpdir(), 'spoor-'));
  const path = join(dir, event.filename || 'document.bin');
  try {
    await writeFile(path, Buffer.from(event.body, event.isBase64Encoded ? 'base64' : 'utf8'));
    const { stdout, stderr, code } = await run(process.env.SPOOR_BIN || '/opt/bin/spoor', [path]);
    return {
      statusCode: code === 0 ? 200 : 422,
      headers: { 'content-type': code === 0 && stdout.trimStart().startsWith('{') ? 'application/json' : 'text/markdown' },
      body: code === 0 ? stdout : stderr,
    };
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
}

function run(command, args) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args);
    let stdout = '';
    let stderr = '';
    child.stdout.on('data', (data) => { stdout += data; });
    child.stderr.on('data', (data) => { stderr += data; });
    child.on('error', reject);
    child.on('close', (code) => resolve({ stdout, stderr, code }));
  });
}
