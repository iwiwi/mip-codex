"""Use the frozen new proof checker with the primary original-model point audit."""
from pathlib import Path
import sys,runpy
r=Path('/home/ubuntu/mip-codex');sys.path.insert(0,str(r/'artifacts/priced-row-subgradient-v1-diagnostic/scripts'));sys.argv=[str(r/'scripts/audit_smoke_solutions.py'),*sys.argv[1:]];runpy.run_path(str(r/'scripts/audit_smoke_solutions.py'),run_name='__main__')
