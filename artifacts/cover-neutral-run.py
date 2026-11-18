from pathlib import Path
import os,sys,json,shutil,subprocess,time,resource,concurrent.futures
root=Path.cwd();os.sched_setaffinity(0,{40});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2)
os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute')
sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256;from validate_solution import validate
out=root/'artifacts/cover-neutral-v1';out.mkdir();spec=json.load(open('artifacts/cover-neutral-source.json'));assert sha256(root/'target/release/mip-codex')==spec['normal_cli_before_sha256'];binary=out/'profile';shutil.copy2(root/'target/release/examples/cover_neutral_profile',binary)
for name,h in spec['sources'].items():
    assert sha256(root/name)==h
    p=out/'source'/name;p.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(root/name,p)
atomic_json(out/'experiment.json',dict(cpu=40,threads=1,memory_bytes=16*1024**3,binary_sha256=sha256(binary),source=spec,seconds_each=60,neutral_rounds=16,neutral_seconds_each=1,audit_cpus=[38,39],target=None,stop_value=None,normal_cli_unchanged_after_example_build=True))
models=[('z26',root/'data/raw/exploratory/z26.mps.gz',root/'artifacts/cover-heap-long/single/result.json'),('sorrell7',root/'data/raw/exploratory/sorrell7.mps.gz',Path('/home/ubuntu/mip-codex/artifacts/spectral-bound-main-primary-audit/sorrell7.mps.gz/result.json'))]
records=[]
for label,model,seed in models:
    d=out/label;d.mkdir();v=validate(model,json.load(open(seed)));atomic_json(d/'seed-validation.json',v);assert v['valid'];cmd=[str(binary),str(model),'60',str(d/'run'),str(seed)];atomic_json(d/'job.json',dict(command=cmd,input_sha256=sha256(model),seed_sha256=sha256(seed)))
    start=time.monotonic();run=subprocess.run(cmd,capture_output=True,text=True,timeout=180);(d/'stdout.log').write_text(run.stdout);(d/'stderr.log').write_text(run.stderr);atomic_json(d/'execution.json',dict(exit_code=run.returncode,wall_seconds=time.monotonic()-start));assert run.returncode==0,run.stderr
    profile=json.load(open(d/'run/profile.json'));assert profile['base_production_fingerprint']=='e5dfb6151bba081e'and profile['base_production_source_directory']==str(root);assert all(r['stats']['validation_failures']==0 for r in profile['neutral_records']);points=sorted((d/'run').glob('parent-*.json'))+[d/'run/result.json'];atomic_json(d/'audit-jobs.json',[dict(point=str(p),sha256=sha256(p))for p in points])
    def worker(shard):
        done=[]
        for k,p in enumerate(points):
            if k%2!=shard:continue
            dest=d/'audit'/p.stem;dest.mkdir(parents=True);cmd=['taskset','-c',str([38,39][shard]),str(root/'.venv/bin/python'),str(root/'scripts/benchmark_validate.py'),str(model),str(p),'/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker',str(dest)]
            with(dest/'audit.log').open('w')as log:r=subprocess.run(cmd,stdout=log,stderr=subprocess.STDOUT,timeout=600)
            assert r.returncode==0,(label,k)
            for kind in ['strict','official']:assert json.load(open(dest/(kind+'-validation.json')))['valid']
            done.append(dict(point=str(p.relative_to(out)),objective=json.load(open(p))['objective'],gmp_valid=True,official_valid=True))
        return done
    validations=[]
    with concurrent.futures.ThreadPoolExecutor(max_workers=2)as pool:
        for result in pool.map(worker,range(2)):validations.extend(result)
    atomic_json(d/'validations.json',validations);result=json.load(open(d/'run/result.json'));record=dict(label=label,seed_count=profile['original_seed_count'],neutral_best=profile['neutral_best'],unique_parents=profile['unique_parents'],parent_outputs=profile['parents'],combined_best=profile['combined']['best'],objective=result['objective'],seconds=profile['seconds'],gmp_valid=len(validations),official_valid=len(validations));records.append(record);atomic_json(out/'records.json',records);print(json.dumps(record),flush=True)
atomic_json(out/'summary.json',dict(models=len(records),output_points_gmp=sum(r['gmp_valid']for r in records),all_official_valid=True))
