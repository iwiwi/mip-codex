from pathlib import Path
import subprocess,hashlib,json
root=Path.cwd();assert not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],text=True)
old=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip();binary=root/'target/release/mip-codex';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();before=sha(binary)
subprocess.run(['git','fetch','origin','main'],check=True);subprocess.run(['git','merge','--ff-only','origin/main'],check=True)
new=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip();assert new=='6b8214962378da04af503ffb7544c85b00da83fd';assert not subprocess.check_output(['git','diff',old,new,'--','src','tests','Cargo.toml','Cargo.lock','build.rs','scripts/benchmark.py','scripts/benchmark_case.py','scripts/benchmark_common.py','scripts/comparison_worker.py','scripts/benchmark_validate.py'],text=True);assert sha(binary)==before=='0bf00779798a657b02581a960261473b3093a21aae08d206b0d180f328a19876'
subprocess.run(['.venv/bin/python','scripts/test_report_benchmark.py'],check=True)
config=root/'artifacts/runs/cover-long-round-final-20261118/config.json';assert sha(config)=='46ffdbd56ef5e38d1a493a96b39f23bc0333dd4c08b2960047dcbf836c9609e9';assert not list((config.parent/'cases').glob('*/summary.json'))
r=dict(old_main=old,new_main=new,production_and_measurement_sources_identical=True,binary_sha256=before,tests_passed=8,final_config_sha256=sha(config),final_snapshot_still_create_only=True,no_final_snapshot_supersession_needed=True,reason='report-only merge; Rust/runtime/evaluation stages unchanged',holdout_individual_quality_not_read=True)
(root/'artifacts/evaluation-report-main-audit/receipt.json').write_text(json.dumps(r,indent=2));print(json.dumps(r))
