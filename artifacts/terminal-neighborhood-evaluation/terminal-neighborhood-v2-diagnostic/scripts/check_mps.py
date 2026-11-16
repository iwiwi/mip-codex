#!/usr/bin/env python3
"""Rust readerと独立HiGHS readerの全行列・境界・目的・整数性を照合。最適化なし。"""
import json
from pathlib import Path
import subprocess
import sys
import time
import highspy
import numpy as np
from scipy.sparse import coo_matrix
from validate_solution import original_lp, matrix

root = Path(__file__).resolve().parents[1]
manifest = json.load(open(root/'data/manifest.json'))
selected = set(sys.argv[1:])
failed = 0
for record in manifest['instances']:
    if selected and record['name'] not in selected:
        continue
    started = time.monotonic()
    try:
        path = root/record['path']
        model = json.loads(subprocess.check_output([str(root/'target/release/mip-codex'),'dump',str(path)],stderr=subprocess.PIPE))
        lp = original_lp(path)
        vs, rs = model['variables'],model['rows']
        assert len(vs)==lp.num_col_ and len(rs)==lp.num_row_, 'dimensions'
        assert [v['name'] for v in vs]==lp.col_names_, 'column names'
        assert [r['name'] for r in rs]==lp.row_names_, 'row names'
        assert model['maximize']==(lp.sense_==highspy.ObjSense.kMaximize), 'sense'
        assert model['offset']==lp.offset_, 'offset'
        for field, expected, inf in [('lower',lp.col_lower_,-np.inf),('upper',lp.col_upper_,np.inf),('objective',lp.col_cost_,0.)]:
            actual=np.array([inf if v[field] is None else v[field] for v in vs])
            np.testing.assert_allclose(actual,expected,rtol=1e-14,atol=1e-14,err_msg='column '+field)
        integers=[v['integer'] for v in vs]
        expected=[v==highspy.HighsVarType.kInteger for v in lp.integrality_] if lp.integrality_ else [False]*len(vs)
        assert integers==expected, 'integrality'
        for field, expected, inf in [('lower',lp.row_lower_,-np.inf),('upper',lp.row_upper_,np.inf)]:
            np.testing.assert_allclose([inf if r[field] is None else r[field] for r in rs],expected,rtol=1e-14,atol=1e-14,err_msg='row '+field)
        ri,ci,data=[],[],[]
        for i,row in enumerate(rs):
            for j,value in row['coefficients']:
                ri.append(i);ci.append(j);data.append(value)
        actual=coo_matrix((data,(ri,ci)),shape=(len(rs),len(vs))).tocsc()
        diff=actual-matrix(lp)
        assert diff.nnz==0 or np.max(np.abs(diff.data))<=1e-14, 'matrix coefficients'
        print(json.dumps(dict(name=record['name'],status='match',seconds=time.monotonic()-started)),flush=True)
    except Exception as error:
        failed+=1
        print(json.dumps(dict(name=record['name'],status='error',error=str(error),seconds=time.monotonic()-started)),flush=True)
print(json.dumps(dict(total_failures=failed)),flush=True)
sys.exit(bool(failed))
