/** Exercise production data code through Exact's actual desktop JS adapter.
 * Only the OS transport is replaced: Git is real, and the folder dialog is scripted. */
import { afterAll, beforeAll, expect, mock, test } from 'bun:test';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { runInThisContext } from 'node:vm';

const execute = promisify(execFile);
const globals = globalThis as Record<string, any>;
const previous = globals.__exact_child_process;
let repo = '', picker = '', pickerError = false;
let answer: any;
beforeAll(async () => {
  repo = mkdtempSync(join(tmpdir(), 'arbor-process-test-'));
  const git = async (...args: string[]) => execute('git', ['-C', repo, ...args]);
  await git('init', '-b', 'main');
  await git('config', 'user.name', 'Native Adapter Test');
  await git('config', 'user.email', 'test@example.com');
  writeFileSync(join(repo, 'first.txt'), 'first\n');
  writeFileSync(join(repo, 'second.txt'), 'second\n');
  await git('add', '.'); await git('commit', '-m', 'A real root commit');
  picker = repo + '\n';
  globals.__exact_process_host = 'macos';
  globals.__exact_process_seam = {
    async request(url: string, init: { body: string }) {
      if (url !== 'exact-process:') throw new Error(`Unexpected transport: ${url}`);
      // Pinned Apple executor_core.rs: the independent lane has 68 MiB
      // plus header allowance; process replies reserve twice their ceiling.
      const ceiling = (init as any).exactIndependentHttp.maxResponseBytes;
      const laneBytes = (4 + 64) * 1024 * 1024 + 2 * 64 * 1024;
      const charge = 2 * ceiling + 2 * 64 * 1024 + Buffer.byteLength(init.body) + 4096;
      if (charge > laneBytes) {
        return { status: 403, text: async () => "native response ceiling exceeds the lane's byte budget" };
      }
      const body = JSON.parse(init.body);
      if (body.kind !== 'execFile') throw new Error(`Unexpected operation: ${body.kind}`);
      let stdout = '', stderr = '', code = 0;
      if (body.program === 'path:osascript') {
        stdout = picker;
        if (pickerError) { code = 1; stderr = 'folder chooser failed'; }
      } else if (body.program === 'path:git') {
        const result = await execute('git', body.args, { maxBuffer: body.maxBuffer, timeout: body.timeout });
        stdout = result.stdout;
      } else throw new Error(`Unexpected program: ${body.program}`);
      return { status: 200, text: async () => JSON.stringify({ stdout, stderr, code, signal: null }) };
    },
  };
  // Install before importing the binding: it captures the SDK adapter at module load.
  runInThisContext(readFileSync(resolve(import.meta.dir, '../.deps/exact/js/src/process.js'), 'utf8'));
  const binding = await import('../.deps/exact/packages/child-process/binding');
  // The SDK bake maps this package's interface manifest to binding.ts.
  mock.module(resolve(import.meta.dir, '../.deps/exact/packages/child-process/exact.json'), () => binding);
  ({ answer } = await import('../apps/exact-client/app'));
});
afterAll(() => {
  mock.restore();
  if (previous === undefined) delete globals.__exact_child_process;
  else globals.__exact_child_process = previous;
  if (repo) rmSync(repo, { recursive: true, force: true });
});

test('production picker, history, detail and patch obey the native process API', async () => {
  pickerError = false; picker = repo + '\n';
  const picked = await answer('pickRepository', []);
  expect(picked.error).toBe('');
  expect(picked.path).toBe(repo);
  const page = await answer('history', [repo, 100, 0]);
  expect(page.error).toBe('');
  expect(page.commits.length).toBe(1);
  const detail = await answer('detail', [repo, page.commits[0].oid]);
  expect(detail.error).toBe('');
  expect(detail.files.length).toBe(2);
  const file = detail.files[0];
  const patch = await answer('filePatch', [repo, detail.oid, file.path, file.oldPath]);
  expect(patch.error).toBe('');
  expect(patch.text).toContain('+first');
});

test('production picker preserves cancellation and reports native process errors', async () => {
  picker = ''; pickerError = false;
  expect(await answer('pickRepository', [])).toEqual({ path: '', error: '' });
  pickerError = true;
  const failed = await answer('pickRepository', []);
  expect(failed.path).toBe('');
  expect(failed.error).toContain('folder chooser failed');
});
