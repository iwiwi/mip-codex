from pathlib import Path
from importlib.util import spec_from_file_location,module_from_spec
from decimal import Decimal
import json,copy,os,subprocess,sys,resource
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
root=Path.cwd();out=root/'artifacts/safe-gmi-box-cert-oracle';out.mkdir();os.sched_setaffinity(0,{40});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));binary=root/'artifacts/safe-gmi-box-diagnostic/bin/mip-codex';spec=spec_from_file_location('checker',root/'artifacts/safe-gmi-box-diagnostic/scripts/verify_cover_cuts.py');checker=module_from_spec(spec);spec.loader.exec_module(checker);records=[]
atomic_json(out/'experiment.json',dict(binary_sha256=sha256(binary),build_info=json.loads(subprocess.check_output([str(binary),'build-info'])),cpu=40,seconds=2,nodes=1,threads=1,memory_bytes=16*1024**3))
for maximize in [False,True]:
 d=out/('max'if maximize else'min');d.mkdir();sense='OBJSENSE\n MAX\n'if maximize else'';cost=1 if maximize else-1;text=f'NAME report\n{sense}ROWS\n N obj\n L r\nCOLUMNS\n x obj {cost}\n x r 2\n y obj {cost}\n y r 2\nRHS\n rhs r 3\nBOUNDS\n BV b x\n BV b y\nENDATA\n';model=d/'model.mps';model.write_text(text);env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(MIP_GOMORY_CUTS='1',MIP_GOMORY_REPORT_ONLY='1',OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1')
 for key in ['BINARY_GRAPH','PARITY_EQUALITIES','COVER_CUTS','MIR_CUTS','AGGREGATE_CUTS','CLIQUE_CUTS','THREE_ROW_CUTS','ROUNDING_CUTS','PRIMAL_REPAIR','LP_DIVING','PRIMAL_PROBE','RINS','FEASIBILITY_PUMP','STRONG_BRANCHING','OBJECTIVE_ROUNDING','OBJECTIVE_DOMAIN_REPORT','ROW_RAY_REPORT','ROW_PACKING_REPORT','ROW_SUBGRADIENT_REPORT','DUAL_GUIDED_PRIMAL']:env['MIP_'+key]='0'
 with(d/'result.json').open('w')as stream,(d/'stderr.log').open('w')as err:p=subprocess.run([str(binary),'solve',str(model),'2','1'],env=env,stdout=stream,stderr=err,timeout=20)
 assert p.returncode==0;raw=(d/'result.json').read_text();result=json.loads(raw);assert result['gomory_bound_proof']is not None;normal=checker.validate_cut_proofs(model,result);decimal=checker.validate_cut_proofs(model,json.loads(raw,parse_float=Decimal));assert normal['valid']is True and decimal['valid']is True,(normal,decimal)
 bad=copy.deepcopy(result);bad['gomory_bound_proof']['normalized_bound']+=100;bad['dual_bound']=(-1 if maximize else 1)*bad['gomory_bound_proof']['normalized_bound'];atomic_json(d/'tampered-bound.json',bad);bad_bound=checker.validate_cut_proofs(model,bad);assert bad_bound['valid']is False,bad_bound
 bad=copy.deepcopy(result);assert bad['gomory_bound_proof']['multipliers'];bad['gomory_bound_proof']['multipliers'][0][1]='7ff0000000000000';atomic_json(d/'tampered-multiplier.json',bad);bad_multiplier=checker.validate_cut_proofs(model,bad);assert bad_multiplier['valid']is False,bad_multiplier
 record=dict(maximize=maximize,normal=normal,decimal=decimal,tampered_bound=bad_bound,tampered_multiplier=bad_multiplier,normalized_bound=result['gomory_bound_proof']['normalized_bound']);records.append(record);atomic_json(d/'validation.json',record)
summary=dict(models=2,valid=2,decimal_valid=2,tampered_rejected=4,failures=0);atomic_json(out/'summary.json',summary);atomic_json(out/'records.json',records);print(summary)
