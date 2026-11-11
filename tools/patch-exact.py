#!/usr/bin/env python3
"""Apply reviewed compatibility fixes to the pinned, ignored Exact checkout."""
from pathlib import Path
import sys

sdk = Path(sys.argv[1])
source = sdk / 'host/apple/Sources/ExactCore/SystemColorNames.swift'
old = 'case "quinaryLabelColor": .quinaryLabelColor'
new = 'case "quinaryLabelColor": .quinaryLabel'
text = source.read_text()
if text.count(old) == 1:
    source.write_text(text.replace(old, new))
    print('Exact: applied NSColor.quinaryLabel Swift-name compatibility fix')
elif text.count(new) != 1:
    raise SystemExit(f'Exact compatibility patch no longer matches {source}; review the SDK pin')
