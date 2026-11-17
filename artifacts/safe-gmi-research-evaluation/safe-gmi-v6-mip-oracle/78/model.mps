NAME gomory
ROWS
 N obj
 L r0
 L r1
 L r2
COLUMNS
 x0 obj 1
 x0 r0 2
 x0 r1 -1
 x0 r2 -1
 x1 obj 0
 x1 r1 5
 x1 r2 3
 x2 obj -1
 x2 r0 2
 x2 r1 5
 x2 r2 1
 x3 obj -7
 x3 r0 3
 x3 r1 -3
 x3 r2 -3
 x4 obj 3
 x4 r1 3
 x4 r2 4
 x5 obj -4
 x5 r0 4
 x5 r1 -4
 x5 r2 -5
RHS
 rhs r0 0
 rhs r1 9
 rhs r2 7
BOUNDS
 LO b x0 -1
 UP b x0 2
 LO b x1 0
 UP b x1 1
 LI b x2 0
 UI b x2 1
 LI b x3 -1
 UI b x3 0
 LI b x4 -1
 UI b x4 2
 LI b x5 -3
 UI b x5 -1
ENDATA
