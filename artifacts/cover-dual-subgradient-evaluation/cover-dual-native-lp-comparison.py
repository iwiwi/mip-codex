from pathlib import Path
import highspy,json,time,hashlib
root=Path.cwd();out=root/'artifacts/cover-dual-native-lp-comparison';out.mkdir();model=root/'data/raw/benchmark/neos-5052403-cygnet.mps.gz';h=highspy.Highs();h.setOptionValue('output_flag',False);assert h.readModel(str(model))!=highspy.HighsStatus.kError;lp=h.getLp();lp.integrality_=[];h.clear();h.passModel(lp)
for k,v in [('output_flag',False),('threads',1),('time_limit',60.),('random_seed',0),('solver','simplex')]:assert h.setOptionValue(k,v)!=highspy.HighsStatus.kError
start=time.monotonic();h.run();info=h.getInfo();solution=h.getSolution();record=dict(purpose='comparison only: native continuous LP relaxation, never a MILP point or own-solver delegated optimization',input_sha256=hashlib.sha256(model.read_bytes()).hexdigest(),version=h.version(),status=str(h.getModelStatus()),objective=h.getObjectiveValue(),wall=time.monotonic()-start,primal_solution_status=int(info.primal_solution_status),dual_solution_status=int(info.dual_solution_status));(out/'summary.json').write_text(json.dumps(record,indent=2)+'\n');
if solution.value_valid:(out/'lp-point.json').write_text(json.dumps(dict(objective=h.getObjectiveValue(),values=dict(zip(lp.col_names_,solution.col_value))),indent=2)+'\n')
print(record)
