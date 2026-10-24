#!/usr/bin/env python3
"""Package canonical DESIGN.md components and CSS; never package prototype fixtures."""
from pathlib import Path
import shutil
root=Path(__file__).resolve().parents[1]
source=root/'gdd/design'
dest=root/'desktop/ui'
for name in ['styles/tokens.css','styles/app.css','js/components.js']:
    out=dest/'design'/name
    out.parent.mkdir(parents=True,exist_ok=True)
    shutil.copy2(source/name,out)
shutil.copytree(source/'assets',dest/'assets',dirs_exist_ok=True)
