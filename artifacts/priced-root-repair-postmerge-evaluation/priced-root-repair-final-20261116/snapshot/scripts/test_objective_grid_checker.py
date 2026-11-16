#!/usr/bin/env python3
import copy,json,tempfile,unittest,shutil,sys
from unittest.mock import patch
from benchmark_common import sha256
import benchmark_case
from pathlib import Path
from verify_cover_cuts import validate_cut_proofs
MODEL='NAME grid\nROWS\n N obj\n E r\n E s\nCOLUMNS\n x obj 0 r 2\n y r -1 s 3\n z obj 6 s -1\nRHS\n rhs r 1 s 2\nBOUNDS\n BV b x\n LO b y 0\n LO b z 0\nENDATA\n'
PROOF=dict(offset=0,step=6,implied=[dict(variable=1,source_row=0,source_name='r'),dict(variable=2,source_row=1,source_name='s')])
class ObjectiveGrid(unittest.TestCase):
    def check(self,model,proof):
        with tempfile.TemporaryDirectory()as directory:
            p=Path(directory)/'model.mps';p.write_text(model)
            return validate_cut_proofs(p,dict(cut_proofs=[],objective_grid=proof))
    def test_chain_and_missing_foundation(self):
        self.assertTrue(self.check(MODEL,PROOF)['valid'])
        bad=copy.deepcopy(PROOF);bad['implied']=bad['implied'][1:];self.assertFalse(self.check(MODEL,bad)['valid'])
        bad=copy.deepcopy(PROOF);bad['implied'].reverse();self.assertFalse(self.check(MODEL,bad)['valid'])
    def test_grid_and_decimal_provenance(self):
        for key,value in [('step',3),('step',0),('offset',1)]:
            bad=copy.deepcopy(PROOF);bad[key]=value;self.assertFalse(self.check(MODEL,bad)['valid'])
        self.assertFalse(self.check(MODEL.replace('z obj 6','z obj 6.000000000000000000000001'),PROOF)['valid'])
        self.assertFalse(self.check(MODEL.replace('rhs r 1','rhs r 1.00000000000000000000001'),PROOF)['valid'])
        self.assertFalse(self.check(MODEL.replace('r -1','r -2'),PROOF)['valid'])
    def test_objname_selects_objective_and_its_offset(self):
        model=MODEL.replace('ROWS','OBJNAME selected\nROWS').replace(' N obj',' N obj\n N selected').replace(' z obj 6',' z obj 0\n z selected 6').replace('rhs r 1','rhs selected -3\n rhs r 1')
        proof=copy.deepcopy(PROOF);proof['offset']=3
        self.assertTrue(self.check(model,proof)['valid'])
    def test_objective_proof_is_checked_without_cuts_or_primal(self):
        bad=copy.deepcopy(PROOF);bad['step']=7
        result=self.check(MODEL,bad);self.assertFalse(result['valid']);self.assertFalse(result['objective_grid_valid'])
    def test_objective_gap_uses_original_integer_box_endpoints(self):
        proof=copy.deepcopy(PROOF);proof['min_domain']=dict(base=0,first_increment=6)
        self.assertTrue(self.check(MODEL,proof)['valid'])
        for key,value in [('base',1),('first_increment',5)]:
            bad=copy.deepcopy(proof);bad['min_domain'][key]=value;self.assertFalse(self.check(MODEL,bad)['valid'])
        bad=copy.deepcopy(PROOF);bad['max_domain']=dict(base=0,first_increment=6)
        self.assertFalse(self.check(MODEL,bad)['valid'])
    def test_bad_objective_proof_blocks_a_global_claim_without_cuts_or_point(self):
        with tempfile.TemporaryDirectory()as directory:
            root=Path(directory);(root/'snapshot/bin').mkdir(parents=True);(root/'snapshot/scripts').mkdir()
            model=root/'model.mps';model.write_text(MODEL)
            result=dict(status='infeasible',objective=None,dual_bound=0,gap=None,values=None,cut_proofs=[],objective_grid=dict(PROOF,step=7))
            solver=root/'snapshot/bin/mip-codex';solver.write_text('#!'+sys.executable+'\nprint('+repr(json.dumps(result))+')\n');solver.chmod(0o755)
            for name in ['verify_cover_cuts.py','benchmark_common.py']:shutil.copyfile(Path(__file__).parent/name,root/'snapshot/scripts'/name)
            config=dict(jobs=[dict(id='case-rust',solver='rust',cpu=0,instance_index=0)],instances=[dict(name='test',split='development',absolute_path=str(model),sha256=sha256(model))],python=sys.executable,memory_bytes=1<<30,seconds=1,read_write_grace_seconds=2,validation_seconds=5)
            path=root/'config.json';path.write_text(json.dumps(config))
            with patch.object(benchmark_case,'cgroup_stats',return_value={}):benchmark_case.run(path,'case-rust')
            summary=json.loads((root/'cases/case-rust/summary.json').read_text())
            self.assertEqual(summary['status'],'invalid_cut_proof');self.assertIsNone(summary['dual_bound']);self.assertFalse(summary['verified_optimal'])
            self.assertFalse(summary['cut_validation']['objective_grid_valid'])
if __name__=='__main__':unittest.main()
