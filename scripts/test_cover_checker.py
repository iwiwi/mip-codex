#!/usr/bin/env python3
import copy,json,shutil,sys,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
from benchmark_common import sha256
from verify_cover_cuts import validate_cut_proofs
import benchmark_case
MODEL='NAME cover\nROWS\n N obj\n L r\nCOLUMNS\n x obj -1 r 2\n y obj -1 r 2\nRHS\n rhs r 3\nBOUNDS\n BV b x\n BV b y\nENDATA\n'
CUT=dict(proof=dict(source_row=0,source_name='r',upper_side=True,cover=[0,1]),row=dict(name='cut',exact_integral=True,lower=None,upper=1,coefficients=[[0,1],[1,1]]))
class Covers(unittest.TestCase):
    def test_valid_and_tampered_certificates(self):
        with tempfile.TemporaryDirectory() as directory:
            p=Path(directory)/'model.mps';p.write_text(MODEL)
            self.assertTrue(validate_cut_proofs(p,dict(cut_proofs=[CUT]))['valid'])
            bad=copy.deepcopy(CUT);bad['row']['upper']=0
            self.assertFalse(validate_cut_proofs(p,dict(cut_proofs=[bad]))['valid'])
            bad=copy.deepcopy(CUT);bad['proof']['cover']=[0,0]
            self.assertFalse(validate_cut_proofs(p,dict(cut_proofs=[bad]))['valid'])
    def test_original_decimal_prevents_a_false_cover(self):
        with tempfile.TemporaryDirectory() as directory:
            p=Path(directory)/'model.mps';p.write_text(MODEL.replace('rhs r 3','rhs r 4.00000000000000000000001'))
            self.assertFalse(validate_cut_proofs(p,dict(cut_proofs=[CUT]))['valid'])
    def test_projection_of_fractional_continuous_box(self):
        model=MODEL.replace(' x obj -1 r 2',' x obj -1 r 0.2').replace(' y obj -1 r 2',' y obj -1 r 0.2\n z r 1').replace('rhs r 3','rhs r 0.4').replace(' BV b y',' BV b y\n LO b z 0.1\n UP b z 1')
        with tempfile.TemporaryDirectory() as directory:
            p=Path(directory)/'model.mps';p.write_text(model)
            self.assertTrue(validate_cut_proofs(p,dict(cut_proofs=[CUT]))['valid'])
            # 厳密にはx=1,y=0,z=.1が可能。丸めによる単独変数cutは不可。
            p.write_text(model.replace('rhs r 0.4','rhs r 0.3'))
            single=copy.deepcopy(CUT);single['proof']['cover']=[0];single['row']['coefficients']=[[0,1]];single['row']['upper']=0
            self.assertFalse(validate_cut_proofs(p,dict(cut_proofs=[single]))['valid'])
            p.write_text(model.replace(' UP b z 1',' PL b z').replace(' z r 1',' z r -1'))
            self.assertFalse(validate_cut_proofs(p,dict(cut_proofs=[CUT]))['valid'])
    def test_floor_division_certificate_and_invalid_domains(self):
        rounding=copy.deepcopy(CUT);rounding['proof'].update(divisor=2,cover=[])
        with tempfile.TemporaryDirectory() as directory:
            p=Path(directory)/'model.mps';p.write_text(MODEL)
            self.assertTrue(validate_cut_proofs(p,dict(cut_proofs=[rounding]))['valid'])
            for bad_model in [MODEL.replace('BV b x','LO b x 0'),MODEL.replace(' BV b x',' LI b x -1'),MODEL.replace('rhs r 3','rhs r 3.0000000000000000001')]:
                p.write_text(bad_model);self.assertFalse(validate_cut_proofs(p,dict(cut_proofs=[rounding]))['valid'])
            p.write_text(MODEL.replace('r 2','r -3',1).replace('r 2','r 2',1).replace('rhs r 3','rhs r -1'))
            negative=copy.deepcopy(rounding);negative['row'].update(coefficients=[[0,-2],[1,1]],upper=-1)
            self.assertTrue(validate_cut_proofs(p,dict(cut_proofs=[negative]))['valid'])
            negative['row']['upper']=0
            self.assertFalse(validate_cut_proofs(p,dict(cut_proofs=[negative]))['valid'])
    def test_empty_cuts_need_no_parser(self):
        self.assertTrue(validate_cut_proofs('/does/not/exist',dict(cut_proofs=[]))['valid'])
    def test_invalid_cut_blocks_infeasible_claim_without_primal(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);(root/'snapshot/bin').mkdir(parents=True);(root/'snapshot/scripts').mkdir()
            p=root/'model.mps';p.write_text(MODEL.replace('rhs r 3','rhs r 4'))
            result=dict(status='infeasible',objective=None,dual_bound=0,gap=None,values=None,cut_proofs=[CUT])
            solver=root/'snapshot/bin/mip-codex';solver.write_text('#!'+sys.executable+'\nprint('+repr(json.dumps(result))+')\n');solver.chmod(0o755)
            for name in ['verify_cover_cuts.py','benchmark_common.py']:
                shutil.copyfile(Path(__file__).parent/name,root/'snapshot/scripts'/name)
            config=dict(jobs=[dict(id='case-rust',solver='rust',cpu=0,instance_index=0)],instances=[dict(name='test',split='development',absolute_path=str(p),sha256=sha256(p))],python=sys.executable,memory_bytes=1<<30,seconds=1,read_write_grace_seconds=2,validation_seconds=5)
            path=root/'config.json';path.write_text(json.dumps(config))
            with patch.object(benchmark_case,'cgroup_stats',return_value={}):benchmark_case.run(path,'case-rust')
            summary=json.loads((root/'cases/case-rust/summary.json').read_text())
            self.assertEqual(summary['status'],'invalid_cut_proof',summary)
            self.assertFalse(summary['verified_optimal']);self.assertIsNone(summary['dual_bound'])
            self.assertFalse(summary['cut_validation']['valid'])
if __name__=='__main__':unittest.main()
