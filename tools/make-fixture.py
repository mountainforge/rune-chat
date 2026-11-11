#!/usr/bin/env python3
"""Create a small deterministic repository for comparing all three interfaces."""
import os
import pathlib
import subprocess
import sys

path = pathlib.Path(sys.argv[1]).resolve()
if path.exists() and any(path.iterdir()):
    raise SystemExit('Choose an empty directory; existing files are never removed.')
path.mkdir(parents=True, exist_ok=True)
env = dict(os.environ, GIT_AUTHOR_DATE='2026-01-01T12:00:00+00:00', GIT_COMMITTER_DATE='2026-01-01T12:00:00+00:00')
def git(*args):
    subprocess.run(['git', '-C', str(path), *args], check=True, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
def commit(message):
    git('add', '.'); git('commit', '--allow-empty', '-m', message)
git('init', '-b', 'main'); git('config', 'user.name', 'Example Author'); git('config', 'user.email', 'author@example.com')
(path / 'README.md').write_text('# A repository to explore\n\nHello, world.\n')
(path / 'src/graph').mkdir(parents=True)
(path / 'src/graph/edges.ts').write_text("export function edgePath(x1: number, x2: number): string {\n  return `M ${x1} 0 L ${x2} 30`;\n}\n")
(path / 'src/graph/GraphRow.tsx').write_text("export const rowHeight = 28;\nexport const laneWidth = 14;\n")
commit('Begin the project\n\nThe root commit adds the first document.')
for i in range(101):
    commit(f'History checkpoint {i + 1}')
git('checkout', '-b', 'feature/graph-renderer')
(path / 'src/graph/edges.ts').write_text("export function edgePath(x1: number, x2: number): string {\n  const mid = 15;\n  if (x1 === x2) {\n    return `M ${x1} 0 L ${x2} 30`;\n  }\n  return `M ${x1} 0 C ${x1} ${mid}, ${x2} ${mid}, ${x2} 30`;\n}\n")
(path / 'src/graph/GraphRow.tsx').write_text("export const rowHeight = 30;\nexport const laneWidth = 18;\nexport const nodeRadius = 4;\n")
(path / 'src/graph/geometry.ts').write_text("export type Point = { x: number; y: number };\n\nexport function laneX(lane: number): number {\n  return 14 + lane * 18;\n}\n\nexport function rowY(row: number): number {\n  return row * 30;\n}\n")
commit('Render merge edges as cubic curves\n\nKeep branch joins smooth at small lane widths and align nodes with the commit rows.')
git('checkout', 'main')
(path / 'README.md').write_text('# A repository to explore\n\nUpdated on the main branch.\n')
commit('Update the project introduction')
git('merge', '--no-ff', 'feature/graph-renderer', '-m', 'Render smooth branch connections\n\nCurved edges keep merges readable at compact lane widths. Inspect each changed file to review the renderer update.')
print(path)
