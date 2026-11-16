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
    def test_mir_mixed_row_and_tampered_original_bounds(self):
        model='NAME mir\nROWS\n N c\n L r\nCOLUMNS\n x c -1 r 3\n t c 1 r -1\nRHS\n rhs r 1\nBOUNDS\n LI b x 0\nENDATA\n'
        cut=dict(proof=dict(source_row=0,source_name='r',upper_side=True,cover=[],divisor=3,mir=dict(upper_bounds=[])),row=dict(coefficients=[[0,2],[1,-1]],lower=None,upper=0,exact_integral=True))
        with tempfile.TemporaryDirectory() as directory:
            p=Path(directory)/'model.mps';p.write_text(model)
            self.assertTrue(validate_cut_proofs(p,dict(cut_proofs=[cut]))['valid'])
            for change in ['rhs','complement','divisor','cover','source']:
                bad=copy.deepcopy(cut)
                if change=='rhs':bad['row']['upper']=1
                elif change=='complement':bad['proof']['mir']['upper_bounds']=[1]
                elif change=='divisor':bad['proof']['divisor']=2
                elif change=='cover':bad['proof']['cover']=[0]
                else:bad['proof']['source_name']='other'
                self.assertFalse(validate_cut_proofs(p,dict(cut_proofs=[bad]))['valid'],change)
            # Integral translation of x can legitimately reproduce the same MIR row.
            p.write_text(model.replace('LI b x 0','LI b x -1'))
            self.assertTrue(validate_cut_proofs(p,dict(cut_proofs=[cut]))['valid'])
            for changed in [model.replace('LI b x 0','LO b x 0'),model.replace('ENDATA',' LO b t -1\nENDATA'),model.replace('rhs r 1','rhs r 1.0000000000000000001')]:
                p.write_text(changed)
                self.assertFalse(validate_cut_proofs(p,dict(cut_proofs=[cut]))['valid'])

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
    def test_rounding_projects_continuous_lower_and_upper_bounds(self):
        cut=copy.deepcopy(CUT);cut['proof'].update(divisor=2,cover=[])
        with tempfile.TemporaryDirectory() as directory:
            p=Path(directory)/'model.mps'
            for coef,bound,bounds in [(1,4,' LO b z 1\n UP b z 2'),(-1,2,' LO b z 0\n UP b z 1')]:
                model=MODEL.replace('RHS',f' z r {coef}\nRHS').replace('rhs r 3',f'rhs r {bound}').replace('ENDATA',bounds+'\nENDATA')
                p.write_text(model);self.assertTrue(validate_cut_proofs(p,dict(cut_proofs=[cut]))['valid'])
                p.write_text(model.replace(' UP b z 1',' PL b z') if coef<0 else model.replace(' LO b z 1',' MI b z'))
                self.assertFalse(validate_cut_proofs(p,dict(cut_proofs=[cut]))['valid'])
    def test_integer_aggregation_cancels_free_variable_and_rejects_corruption(self):
        for lower in [False,True]:
            sign=-1 if lower else 1;kind='G'if lower else'L'
            model=f'NAME aggregate\nROWS\n N obj\n L a\n {kind} b\nCOLUMNS\n x a 2\n z b {2*sign}\n t a 1 b {-sign}\nRHS\n rhs a 2 b {sign}\nBOUNDS\n BV bd x\n BV bd z\n FR bd t\nENDATA\n'
            cut=copy.deepcopy(CUT);cut['proof']=dict(source_row=0,source_name='a',upper_side=True,cover=[],divisor=2,aggregation=[dict(source_row=0,source_name='a',upper_side=True,weight=1),dict(source_row=1,source_name='b',upper_side=not lower,weight=1)])
            with tempfile.TemporaryDirectory()as directory:
                path=Path(directory)/'model.mps';path.write_text(model)
                self.assertTrue(validate_cut_proofs(path,dict(cut_proofs=[cut]))['valid'])
                for mode in range(8):
                    bad=copy.deepcopy(cut);term=bad['proof']['aggregation'][1]
                    if mode==0:term['weight']=0
                    elif mode==1:term['weight']=True
                    elif mode==2:term['weight']=2**52+1
                    elif mode==3:term['source_name']='wrong'
                    elif mode==4:term['source_row']=0
                    elif mode==5:bad['proof']['aggregation'][0]['upper_side']=False
                    elif mode==6:bad['row']['upper']=0
                    else:bad['proof']['aggregation']=None
                    self.assertFalse(validate_cut_proofs(path,dict(cut_proofs=[bad]))['valid'],mode)
                path.write_text(model.replace('x a 2','x a 2.000000000000000000001'))
                self.assertFalse(validate_cut_proofs(path,dict(cut_proofs=[cut]))['valid'])
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
