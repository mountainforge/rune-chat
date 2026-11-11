import { test, expect } from 'bun:test';
import { execFile as exec } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { history, detail, filePatch, patchLines, patchPairs, type Execute } from './git';
const execute = promisify(exec);
const run: Execute = async (program, args) => (await execute(program.replace('path:', ''), args, { maxBuffer: 32 * 1024 * 1024 })).stdout;
async function fixture(fn: (repo: string, git: (...args: string[]) => Promise<string>) => Promise<void>) {
  const path = mkdtempSync(join(tmpdir(), 'git client '));
  const git = (...args: string[]) => run('path:git', ['-C', path, ...args]);
  try { await git('init', '-b', 'main'); await git('config', 'user.name', 'Test Author'); await git('config', 'user.email', 'test@example.com'); await fn(path, git); }
  finally { rmSync(path, { recursive: true, force: true }); }
}
test('empty and invalid repositories and revision injection', async () => fixture(async (p) => {
  expect((await history(run, p, 100)).commits).toEqual([]);
  expect((await history(run, join(p, 'missing'), 100)).error).not.toBe('');
  expect((await detail(run, p, '--help')).error).toBe('Error: Invalid commit ID');
}));
test('root, Unicode, spaces, full message, metadata and patch', async () => fixture(async (p, git) => {
  writeFileSync(join(p, 'hello world.txt'), 'héllo\n'); await git('add', '.'); await git('commit', '-m', 'First 🦀\n\nBody details');
  const page = await history(run, p, 100); expect(page.error).toBe(''); expect(page.commits[0].subject).toBe('First 🦀');
  const d = await detail(run, p, page.commits[0].oid);
  expect(d.files[0].additions).toBe(1); expect(d.author).toBe('Test Author'); expect(d.parents).toBe(''); expect(d.message).toContain('Body details'); expect((await filePatch(run,p,d.oid,d.files[0].path,d.files[0].oldPath)).text).toContain('+héllo');
}));
test('pagination and merge first-parent patch', async () => fixture(async (p, git) => {
  await git('commit', '--allow-empty', '-m', 'Root'); await git('checkout', '-b', 'feature');
  writeFileSync(join(p, 'feature.txt'), 'feature\n'); await git('add', '.'); await git('commit', '-m', 'Feature');
  await git('checkout', 'main'); await git('commit', '--allow-empty', '-m', 'Main'); await git('merge', '--no-ff', 'feature', '-m', 'Merge');
  const page = await history(run, p, 2); expect(page.more).toBe(true); expect(page.commits.length).toBe(2);
  const all = await history(run, p, 100); expect(all.more).toBe(false); expect(all.commits.length).toBe(4);
  const d = await detail(run, p, page.commits[0].oid); expect(d.parents.split(' ').length).toBe(2); expect((await filePatch(run,p,d.oid,d.files[0].path,d.files[0].oldPath)).text).toContain('feature.txt');
}));

test('all branches, graph topology, renamed files and isolated literal path diffs',async()=>fixture(async(p,git)=>{
  writeFileSync(join(p,'a.txt'),'one\n');writeFileSync(join(p,'b.txt'),'two\n');await git('add','.');await git('commit','-m','Root');
  await git('checkout','-b','unmerged');await git('commit','--allow-empty','-m','Only on feature');await git('checkout','main');
  await git('mv','a.txt','renamed.txt');writeFileSync(join(p,'b.txt'),'changed\n');writeFileSync(join(p,':(glob)*'),'literal filename\n');await git('add','.');await git('commit','-m','Rename and modify');
  const h=await history(run,p,100);expect(h.error).toBe('');expect(h.commits.some(c=>c.subject==='Only on feature'&&c.refs.includes('unmerged'))).toBe(true);
  expect(h.commits.every(c=>c.graph.startsWith('data:image/svg+xml,'))).toBe(true);
  const d=await detail(run,p,h.commits.find(c=>c.subject==='Rename and modify')!.oid);expect(d.files.length).toBe(3);
  const renamed=d.files.find(f=>f.status.startsWith('R'))!;expect(renamed.oldPath).toBe('a.txt');expect(renamed.path).toBe('renamed.txt');
  const b=d.files.find(f=>f.path==='b.txt')!;const patch=await filePatch(run,p,d.oid,b.path,b.oldPath);expect(patch.text).toContain('+changed');expect(patch.text).not.toContain('renamed.txt');
  const literal=d.files.find(f=>f.path===':(glob)*')!;expect((await filePatch(run,p,d.oid,literal.path,literal.oldPath)).text).toContain('+literal filename');
}));

test('split rows preserve hunk line numbers and missing sides',()=>{
 const lines=patchLines('@@ -10,2 +10,3 @@\n-old\n+new\n+extra\n same','scope');
 const pairs=patchPairs(lines);expect(pairs[1].left.old).toBe('10');expect(pairs[1].right.next).toBe('10');expect(pairs[2].left.kind).toBe(5);expect(pairs[2].right.text).toBe('extra');expect(pairs[3].right.next).toBe('12');
});
