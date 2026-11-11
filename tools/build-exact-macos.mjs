/** Reuse only a native bundle whose SDK source receipts are still current. */
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
const root = resolve(fileURLToPath(new URL('..', import.meta.url)));
const sdk = resolve(root, '.deps/exact');
const { resolveApp } = await import(`${sdk}/scripts/app.mjs`);
const { nativeBuild } = await import(`${sdk}/scripts/agent-build.mjs`);
const state = nativeBuild('macos', resolveApp(resolve(root, 'apps/exact-client')));
if (!state.missing && state.changed.length === 0) {
  console.log(`Reusing native bundle: SDK source receipts are current (${state.built})`);
} else {
  console.log(state.missing ? 'Native bundle is missing.' : `Native build inputs changed: ${state.changed.join(', ')}`);
  if (!process.argv.includes('--check-only')) {
    if (process.platform !== 'darwin') throw new Error('Native AppKit builds require macOS.');
    const child = Bun.spawn(['bash', resolve(root, 'tools/exact.sh'), 'mac', '--bundle'], { cwd: root, env: process.env, stdout: 'inherit', stderr: 'inherit' });
    process.exitCode = await child.exited;
  }
}
