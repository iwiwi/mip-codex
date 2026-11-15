#!/usr/bin/env python3
"""開発用MIPLIBの根LP目的を独立HiGHS relaxationと照合する。"""
import json
import os
from pathlib import Path
import subprocess
import sys
import highspy
from validate_solution import original_lp

os.environ['MIP_PROPAGATION'] = '0'  # compare original continuous LP relaxations
root=Path(__file__).resolve().parents[1]
manifest={r['name']:r for r in json.load(open(root/'data/manifest.json'))['instances']}
failed=0
for name in sys.argv[1:]:
    record=manifest[name]
    assert record['split']=='development'
    try:
        path=root/record['path']
        lp=original_lp(path);lp.integrality_=[]
        h=highspy.Highs();h.setOptionValue('output_flag',False);h.setOptionValue('threads',1);h.setOptionValue('time_limit',30.);h.setOptionValue('small_matrix_value',1e-12)
        h.passModel(lp);h.run()
        assert h.getModelStatus()==highspy.HighsModelStatus.kOptimal,str(h.getModelStatus())
        expected=h.getObjectiveValue()
        env=os.environ.copy();env['MIP_LP_ENGINE']='revised'
        result=json.loads(subprocess.check_output([str(root/'target/release/mip-codex'),'solve',str(path),'30','1'],env=env,timeout=40))
        assert result['status'] in ('optimal','node_limit'),result
        actual=result['dual_bound'];assert actual is not None,result
        assert abs(actual-expected)<=1e-6*(1+abs(expected)),(expected,result)
        print(json.dumps(dict(name=name,status='pass',highs=expected,revised=actual,nodes=result['nodes'],pivots=result['lp_pivots'],seconds=result['seconds'],input_sha256=record['sha256'])),flush=True)
    except Exception as error:
        failed+=1
        print(json.dumps(dict(name=name,status='fail',error=str(error))),flush=True)
print(json.dumps(dict(cases=len(sys.argv)-1,failures=failed)),flush=True)
sys.exit(bool(failed))
