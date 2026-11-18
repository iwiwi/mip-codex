from pathlib import Path
import os,sys,json,shutil,subprocess,time,resource
root=Path.cwd();os.sched_setaffinity(0,{40});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2)
os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute')
sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256;from validate_solution import validate
out=root/'artifacts/cover-parent-lns-v1';out.mkdir();spec=json.load(open('artifacts/cover-parent-lns-source.json'));assert sha256(root/'target/release/mip-codex')==spec['normal_cli_before_sha256'];binary=out/'profile';shutil.copy2(root/'target/release/examples/cover_parent_lns_profile',binary)
for name,h in spec['sources'].items():
    assert sha256(root/name)==h
    p=out/'source'/name;p.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(root/name,p)
model=root/'data/raw/exploratory/z26.mps.gz';prior=root/'artifacts/cover-neutral-v1/z26';job=json.load(open(prior/'job.json'));assert job['input_sha256']==sha256(model);old={r['point']:r['sha256']for r in json.load(open(prior/'audit-jobs.json'))};parents=sorted((prior/'run').glob('parent-*.json'));assert len(parents)==17;parent_records=[]
for p in parents:
    assert sha256(p)==old[str(p)]
    strict=json.load(open(prior/'audit'/p.stem/'strict-validation.json'));official=json.load(open(prior/'audit'/p.stem/'official-validation.json'));assert strict['valid']and official['valid'];assert strict['checker_sha256']==sha256('/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute');parent_records.append(dict(path=str(p),sha256=sha256(p),prior_gmp_valid=True,prior_official_valid=True,validation_reused_by_identical_point_input_checker_sha=True))
cmd=[str(binary),str(model),'600',str(out/'run'),*map(str,parents)]
atomic_json(out/'experiment.json',dict(cpu=40,threads=1,memory_bytes=16*1024**3,binary_sha256=sha256(binary),source=spec,input_sha256=sha256(model),parents=parent_records,command=cmd,global_seconds=600,max_trials=60,seconds_per_trial=10,nodes_per_trial=2000000,free_limit=2048,target=None,stop_value=None,normal_cli_unchanged_after_example_build=True,purpose='own neutral parents + current best restricted triplet unions; primal only; no original global bound transfer'))
start=time.monotonic();run=subprocess.run(cmd,capture_output=True,text=True,timeout=900);(out/'stdout.log').write_text(run.stdout);(out/'stderr.log').write_text(run.stderr);atomic_json(out/'execution.json',dict(exit_code=run.returncode,wall_seconds=time.monotonic()-start));assert run.returncode==0,run.stderr
profile=json.load(open(out/'run/profile.json'));assert profile['base_production_fingerprint']=='e5dfb6151bba081e'and profile['base_production_source_directory']==str(root);records=[]
for p in sorted((out/'run').glob('*-original.json'))+[out/'run/result.json']:
    value=json.load(open(p));v=validate(model,value);atomic_json(p.with_name(p.stem+'-validation.json'),v);assert v['valid'];records.append(dict(kind='original',path=str(p.relative_to(out)),objective=value['objective'],gmp_valid=True))
for mps in sorted((out/'run').glob('*-restricted.mps')):
    p=mps.with_suffix('.json');value=json.load(open(p));v=validate(mps,value);atomic_json(p.with_name(p.stem+'-validation.json'),v);assert v['valid'];records.append(dict(kind='restricted',path=str(p.relative_to(out)),objective=value['objective'],gmp_valid=True))
atomic_json(out/'validations.json',records);summary=dict(initial=profile['initial'],best=profile['best'],improvements=profile['improvements'],trials=len(profile['records']),seconds=profile['seconds'],output_original_gmp=sum(r['kind']=='original'for r in records),output_restricted_gmp=sum(r['kind']=='restricted'for r in records),parents_reused_identical_sha=17,global_bound_or_optimality_claim=False);atomic_json(out/'summary.json',summary);print(json.dumps(summary),flush=True)
