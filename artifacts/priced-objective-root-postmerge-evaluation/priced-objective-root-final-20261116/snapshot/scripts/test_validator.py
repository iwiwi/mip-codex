#!/usr/bin/env python3
import copy
import json
from decimal import Decimal
import unittest
from pathlib import Path
from validate_solution import validate
import tempfile

MPS='''NAME check
ROWS
 N cost
 L cap
COLUMNS
 x cost 2 cap 1
 y cost 1 cap 1
RHS
 rhs cost -3 cap 4
BOUNDS
 BV b x
 UP b y 5
ENDATA
'''
class ValidatorTests(unittest.TestCase):
    def test_detects_each_failure(self):
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)/'original.mps';path.write_text(MPS)
            solution=dict(objective=7., values=dict(x=1.,y=2.))
            self.assertTrue(validate(path,solution)['valid'])
            for kind,value in [('integer',0.5),('bound',2.),('row',5.)]:
                changed=copy.deepcopy(solution)
                changed['values']['y' if kind=='row' else 'x']=value
                changed['objective']=3+2*changed['values']['x']+changed['values']['y']
                check=validate(path,changed)
                self.assertFalse(check['valid'],kind)
                json.dumps(check,allow_nan=False)
            solution['objective']=8.
            self.assertFalse(validate(path,solution)['valid'])
            solution['values'].pop('y')
            with self.assertRaises(ValueError):validate(path,solution)
    def test_exact_cancellation(self):
        text = MPS.replace('x cost 2 cap 1', 'x cost 0 cap 100000000').replace('y cost 1 cap 1', 'y cost 1 cap 1').replace('rhs cost -3 cap 4', 'rhs cap 10000000000000000').replace('BV b x', 'FX b x 100000000').replace('UP b y 5', 'BV b y')
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'cancel.mps'
            path.write_text(text)
            result = validate(path, dict(objective=1., values=dict(x=1e8,y=1.)))
            self.assertTrue(result['float_diagnostics']['valid'])
            self.assertFalse(result['valid'])
            self.assertEqual(result['exact']['row_violation'], '1')
            result = validate(path, dict(objective=0, values=dict(x=Decimal('100000000.0000000001'), y=0)))
            self.assertTrue(result['float_diagnostics']['valid'])
            self.assertFalse(result['valid'])
            self.assertEqual(result['exact']['row_violation'], '1/100')

    def test_long_decimal_and_fortran_exponents(self):
        text = MPS.replace('x cost 2 cap 1', 'x cost 2D0 cap 0.0100000000000000000000000000000000000000000000000000D0').replace('rhs cost -3 cap 4', 'rhs cost -3D0 cap 2.5')
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'long.mps'
            path.write_text(text)
            self.assertTrue(validate(path, dict(objective=7.,values=dict(x=1.,y=2.)))['valid'])

if __name__=='__main__':unittest.main()
