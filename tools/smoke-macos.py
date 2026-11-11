#!/usr/bin/env python3
"""Check that the packaged Rust apps launch with a real repository on macOS."""
import os
import subprocess
import sys
from pathlib import Path

repo = str(Path(sys.argv[1]).resolve())
for kind, title in [('gpui', 'GPUI'), ('rune', 'Rune')]:
    binary = Path(f'dist/Arbor {title}.app/Contents/MacOS/git-{kind}')
    log = Path(f'dist/{kind}-launch.log')
    with log.open('w') as output:
        process = subprocess.Popen([str(binary.resolve())], env={**os.environ, 'GIT_CLIENT_REPO': repo}, stdout=output, stderr=output)
        try:
            code = process.wait(timeout=10)
            raise SystemExit(f'{kind} exited during startup ({code}): {log.read_text()}')
        except subprocess.TimeoutExpired:
            print(f'{kind}: remained running for 10 seconds with a real repository')
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
