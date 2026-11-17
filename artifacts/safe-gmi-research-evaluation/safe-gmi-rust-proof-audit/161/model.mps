NAME gomory
ROWS
 N obj
 L r0
 L r1
 L r2
COLUMNS
 x0 obj 0
 x0 r0 -2
 x0 r1 -2
 x0 r2 3
 x1 obj 0
 x1 r0 2
 x1 r1 -4
 x1 r2 -3
 x2 obj 0
 x2 r0 1
 x2 r1 -1
 x2 r2 1
 x3 obj 0
 x3 r1 -2
RHS
 rhs r0 2
 rhs r1 -10
 rhs r2 2
BOUNDS
 LO b x0 1
 UP b x0 2
 LO b x1 1
 UP b x1 4
 LI b x2 -2
 UI b x2 0
 LI b x3 -1
 UI b x3 1
ENDATA
