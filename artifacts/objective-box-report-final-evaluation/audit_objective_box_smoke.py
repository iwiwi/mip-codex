import sys,runpy
from pathlib import Path
root=Path('/home/ubuntu/mip-codex')
sys.path.insert(0,str(root/'artifacts/objective-box-report-default-diagnostic/scripts'))
import verify_cover_cuts
sys.path.insert(0,str(root/'scripts'))
runpy.run_path(str(root/'scripts/audit_smoke_solutions.py'),run_name='__main__')
