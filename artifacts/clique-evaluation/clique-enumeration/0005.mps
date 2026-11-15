NAME graph
ROWS
 N obj
 L r0
 L r1
 L r2
COLUMNS
 x0 obj -2
 x0 r0 -1
 x0 r1 -1
 x1 obj 4
 x1 r0 1
 x1 r2 1
 x2 obj 3
 x2 r1 1
 x2 r2 1
 c0 r0 1
 c1 r1 1
 c2 r2 1
RHS
 rhs r0 -0.25
 rhs r1 -0.25
 rhs r2 0.75
BOUNDS
 BV b x0
 BV b x1
 BV b x2
 LO b c0 -0.25
 UP b c0 0.75
 LO b c1 -0.25
 UP b c1 0.75
 LO b c2 -0.25
 UP b c2 0.75
ENDATA
