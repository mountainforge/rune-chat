#!/usr/bin/env python3
"""Assemble relocatable, ad-hoc signed macOS preview bundles."""
import argparse
import os
import platform
import plistlib
import shutil
import subprocess
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('framework', choices=['gpui', 'rune', 'exact'])
parser.add_argument('--exact-app', type=Path)
args = parser.parse_args()
if platform.system() != 'Darwin':
    parser.error('Run packaging on macOS after building the native executable.')
root = Path(__file__).resolve().parent.parent
out = root / 'dist'
out.mkdir(exist_ok=True)
name = f'Arbor {args.framework.upper() if args.framework == "gpui" else args.framework.title()}'
bundle = out / f'{name}.app'
if bundle.exists():
    raise SystemExit(f'Refusing to replace existing bundle: {bundle}')
if args.framework == 'exact':
    if not args.exact_app or not (args.exact_app / 'Contents/Info.plist').is_file():
        parser.error('--exact-app must name the bundle produced by Exact')
    # Keep the SDK-generated resources, native modules and signing entitlements.
    subprocess.run(['ditto', str(args.exact_app), str(bundle)], check=True)
else:
    contents = bundle / 'Contents'
    (contents / 'MacOS').mkdir(parents=True)
    resources = contents / 'Resources'
    resources.mkdir()
    executable = f'git-{args.framework}'
    shutil.copy2(root / 'target/debug' / executable, contents / 'MacOS' / executable)
    shutil.copytree(root / 'assets/fonts', resources / 'font-licenses')
    with (contents / 'Info.plist').open('wb') as stream:
        plistlib.dump({
            'CFBundleName': name,
            'CFBundleDisplayName': name,
            'CFBundleExecutable': executable,
            'CFBundleIdentifier': f'dev.runechat.git.{args.framework}',
            'CFBundlePackageType': 'APPL',
            'CFBundleShortVersionString': '0.1.0',
            'CFBundleVersion': os.environ.get('GITHUB_RUN_NUMBER', '1'),
            'LSMinimumSystemVersion': '14.0',
            'NSHighResolutionCapable': True,
            'NSSupportsAutomaticGraphicsSwitching': True,
        }, stream)
    subprocess.run(['codesign', '--force', '--sign', '-', str(bundle)], check=True)
subprocess.run(['codesign', '--verify', '--deep', '--strict', str(bundle)], check=True)
arch = platform.machine()
archive = out / f'Arbor-{args.framework}-macOS-{arch}.zip'
subprocess.run(['ditto', '-c', '-k', '--sequesterRsrc', '--keepParent', str(bundle), str(archive)], check=True)
print(archive)
