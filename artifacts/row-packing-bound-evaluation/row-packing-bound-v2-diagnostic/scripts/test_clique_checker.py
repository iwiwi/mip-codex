#!/usr/bin/env python3
import copy,tempfile,unittest
from pathlib import Path
from verify_cover_cuts import validate_cut_proofs
MODEL='NAME triangle\nROWS\n N obj\n L a\n L b\n L c\nCOLUMNS\n x obj -1 a 1\n x b 1\n y obj -1 a 1\n y c 1\n z obj -1 b 1\n z c 1\nRHS\n rhs a 1 b 1\n rhs c 1\nBOUNDS\n BV bd x\n BV bd y\n BV bd z\nENDATA\n'
CUT=dict(row=dict(name='clique',coefficients=[[0,1],[1,1],[2,1]],lower=None,upper=1,exact_integral=True),proof=dict(source_row=0,source_name='a',upper_side=True,cover=[0,1,2],clique=[dict(source_row=0,source_name='a',upper_side=True,variables=[0,1]),dict(source_row=1,source_name='b',upper_side=True,variables=[0,2]),dict(source_row=2,source_name='c',upper_side=True,variables=[1,2])]))
class Cliques(unittest.TestCase):
 def check(self,model,cut):
  with tempfile.TemporaryDirectory()as d:
   p=Path(d)/'model.mps';p.write_text(model);return validate_cut_proofs(p,dict(cut_proofs=[cut]))
 def test_positive_and_negative_literals(self):
  self.assertTrue(self.check(MODEL,CUT)['valid'])
  model=MODEL.replace('x obj -1 a 1','x obj -1 a -1').replace('x b 1','x b -1').replace('rhs a 1 b 1','rhs a 0 b 0');cut=copy.deepcopy(CUT);cut['row']['coefficients'][0][1]=-1;cut['row']['upper']=0
  self.assertTrue(self.check(model,cut)['valid'])
 def test_corruption_is_rejected(self):
  for change in range(13):
   c=copy.deepcopy(CUT)
   if change==0:c['proof']['clique'].pop()
   elif change==1:c['proof']['clique'][0]['source_row']=999
   elif change==2:c['proof']['clique'][0]['source_name']='bad'
   elif change==3:c['proof']['clique'][0]['upper_side']=False
   elif change==4:c['proof']['clique'][0]['variables']=[0,0]
   elif change==5:c['proof']['clique'][1]=copy.deepcopy(c['proof']['clique'][0])
   elif change==6:c['row']['coefficients'][0][1]=-1
   elif change==7:c['row']['upper']=0
   elif change==8:c['proof']['cover'].reverse()
   elif change==9:c['proof']['divisor']=2
   elif change==10:c['proof']['clique'][0]['variables']=[0,True]
   elif change==11:c['row']['coefficients'][0][1]=0
   else:c['row']['exact_integral']=False
   self.assertFalse(self.check(MODEL,c)['valid'],change)
 def test_missing_contradiction_and_nonbinary_literal_are_rejected(self):
  self.assertFalse(self.check(MODEL.replace('rhs c 1','rhs c 2'),CUT)['valid'])
  self.assertFalse(self.check(MODEL.replace(' BV bd x',' UP bd x 1'),CUT)['valid'])
if __name__=='__main__':unittest.main()
