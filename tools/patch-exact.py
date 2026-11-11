#!/usr/bin/env python3
"""Apply reviewed compatibility fixes to the pinned, ignored Exact checkout."""
from pathlib import Path
import sys

sdk = Path(sys.argv[1])
patches = [
    ('host/apple/Sources/ExactCore/SystemColorNames.swift',
     'case "quinaryLabelColor": .quinaryLabelColor',
     'case "quinaryLabelColor": .quinaryLabel',
     'NSColor.quinaryLabel Swift name'),
    ('host/apple/Sources/ExactKit/FrameHealthCard.swift',
     'let figures = windows.map { figures(',
     'let figures = windows.map { Self.figures(',
     'frame-health method/local-variable shadowing'),
]
for relative, old, new, description in patches:
    source = sdk / relative
    text = source.read_text()
    if text.count(old) == 1:
        source.write_text(text.replace(old, new))
        print(f'Exact: applied {description} compatibility fix')
    elif text.count(new) != 1:
        raise SystemExit(f'Exact compatibility patch no longer matches {source}; review the SDK pin')
