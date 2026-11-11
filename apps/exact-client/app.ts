import type { Answer, Sources } from './app.contract.d.ts';
import { execFile } from '@exact/child-process';
import { history, detail, filePatch, type Execute } from './git';
export const appId = 'dev.runechat.git.exact';
export const grants = 'process.spawn path:git\nprocess.spawn path:osascript';
const run: Execute = async (program, args) => {
  const r = await execFile(program, args, { encoding: 'utf8', timeout: 30000, maxBuffer: 16 * 1024 * 1024 });
  if (r.code !== 0) throw new Error(String(r.stderr).trim() || `Git exited with ${r.code}`);
  return String(r.stdout);
};
const sources: Sources = {
  history: ([repo, limit]) => history(run, repo, limit),
  detail: ([repo, oid]) => detail(run, repo, oid),
  filePatch: ([repo, oid, path, oldPath]) => filePatch(run, repo, oid, path, oldPath),
  pickRepository: async () => {
    try {
      const path = await run('path:osascript', ['-e', 'try', '-e', 'POSIX path of (choose folder with prompt "Open Git repository")', '-e', 'on error number -128', '-e', 'return ""', '-e', 'end try']);
      return { path: path.replace(/\r?\n$/, ''), error: '' };
    } catch (e) { return { path: '', error: String(e) }; }
  },
};
export const answer: Answer = (source, args, store, storage, native) => sources[source](args, store, storage, native);
