/** Drive the unchanged production Mac app with real Git. Only the folder dialog is scripted. */
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { history, detail, filePatch, type Execute } from '../apps/exact-client/git';

const root = resolve(import.meta.dir, '..');
const repo = resolve(process.argv[2] ?? '/tmp/git-client-demo');
const execute = promisify(execFile);
const run: Execute = async (program, args) => (await execute(program.replace(/^path:/, ''), args, { maxBuffer: 32 * 1024 * 1024 })).stdout;
const page = await history(run, repo, 1000);
if (page.error || page.commits.length < 101) throw new Error(page.error || 'Create the fixture with tools/make-fixture.py first.');
const first = await detail(run, repo, page.commits[0].oid);
if (first.error || first.files.length < 2) throw new Error(first.error || 'The fixture needs two changed files.');
let other = '', otherPath = '', empty = '';
for (const c of page.commits.slice(1, 12)) {
  if (!(await detail(run, repo, c.oid)).files.length) { empty = c.oid; break; }
}
const rootCommit = page.commits[page.commits.length - 1];
const rootDetail = await detail(run, repo, rootCommit.oid);
const rootFile = rootDetail.files[0];
if (!empty || !rootFile) throw new Error('The fixture needs an empty commit and a non-empty root.');
const rootPatch = await filePatch(run, repo, rootCommit.oid, rootFile.path, rootFile.oldPath);
for (const c of page.commits.slice(1, 12)) {
  const d = await detail(run, repo, c.oid);
  if (d.files.length && d.files[0].path !== first.files[0].path) { other = c.oid; otherPath = d.files[0].path; break; }
}
if (!other) throw new Error('The fixture needs another non-empty commit.');
const second = first.files[1];
const patch = await filePatch(run, repo, first.oid, second.path, second.oldPath);
if (patch.error) throw new Error(patch.error);
const temporary = mkdtempSync(join(tmpdir(), 'arbor-exact-native-'));
const test = join(temporary, 'native.test.contract');
const picker = join(temporary, 'picker.json');
writeFileSync(picker, JSON.stringify({ execFile: { stdout: repo + '\n', stderr: '', code: 0, after: 0 } }));
writeFileSync(test, `test "native Git workspace opens, selects files and commits, changes diff mode and loads history"
  size 1440x900
  tap "open"
  clock data
  expect state openError == ""
  expect state repo == ${JSON.stringify(repo)}
  expect text "count" == "100 commits"
  expect text "file-path" == ${JSON.stringify(first.files[0].path)}
  tap "split"
  expect state split == true
  tap "unified"
  expect state split == false
  tap "file-1"
  clock data
  expect text "file-path" == ${JSON.stringify(second.path)}
  expect state patch.text == ${JSON.stringify(patch.text)}
  tap "commit-${other}"
  clock data
  expect text "file-path" == ${JSON.stringify(otherPath)}
  tap "commit-${empty}"
  clock data
  expect text "file-path" == "No changed files"
  tap "more"
  clock data
  expect text "count" == "${page.commits.length} commits"
  tap "commit-${rootCommit.oid}"
  clock data
  expect text "file-path" == ${JSON.stringify(rootFile.path)}
  expect state patch.text == ${JSON.stringify(rootPatch.text)}
  tap "refresh"
  clock data
  expect text "count" == "100 commits"
  expect text "file-path" == ${JSON.stringify(first.files[0].path)}
`);
try {
  const checkOnly = process.argv.includes('--check-only');
  if (!checkOnly && process.platform !== 'darwin') throw new Error('Native AppKit execution requires macOS; use --check-only to validate the test here.');
  const args = checkOnly
    ? ['contract', 'test', test]
    : ['test', 'macos', test, '--process', 'path:git=real', '--process', `path:osascript=script:${picker}`];
  const child = Bun.spawn([process.execPath, 'exact.mjs', ...args], {
    cwd: join(root, 'apps/exact-client'),
    env: { ...process.env, EXACT2: join(root, '.deps/exact'), EXACT_FEEDBACK: '0' },
    stdout: 'inherit', stderr: 'inherit',
  });
  if (await child.exited) {
    process.exitCode = 1;
    if (!checkOnly) {
      console.error('Native test failed; collecting production state and host diagnostics.');
      const diagnostic = Bun.spawn([process.execPath, 'exact.mjs', 'agent', 'macos',
        '--process', 'path:git=real', '--process', `path:osascript=script:${picker}`,
        'tap open', 'clock data', 'state', 'logs --all', 'tree'], {
        cwd: join(root, 'apps/exact-client'),
        env: { ...process.env, EXACT2: join(root, '.deps/exact'), EXACT_FEEDBACK: '0' },
        stdout: 'inherit', stderr: 'inherit',
      });
      await diagnostic.exited;
    }
  }
} finally {
  rmSync(temporary, { recursive: true, force: true });
}
