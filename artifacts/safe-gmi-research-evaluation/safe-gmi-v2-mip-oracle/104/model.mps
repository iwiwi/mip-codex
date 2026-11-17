NAME gomory
ROWS
 N obj
 L r0
 L r1
COLUMNS
 x0 obj -3
 x0 r0 -4
 x0 r1 2
 x1 obj 0
 x1 r0 3
 x1 r1 4
 x2 obj 0
 x2 r0 4
 x2 r1 -2
 x3 obj 4
 x3 r0 -1
 x3 r1 -5
RHS
 rhs r0 -3
 rhs r1 8
BOUNDS
 LI b x0 0
 UI b x0 2
 LI b x1 -1
 UI b x1 1
 LO b x2 -1
 UP b x2 1
 LI b x3 -1
 UI b x3 0
ENDATA
