NAME gomory
OBJSENSE
 MAX
ROWS
 N obj
 L r0
 L r1
 L r2
 L r3
COLUMNS
 x0 obj -1
 x0 r0 4
 x0 r1 4
 x0 r2 -4
 x0 r3 3
 x1 obj 0
 x1 r0 5
 x1 r1 5
 x1 r2 -2
 x1 r3 -4
 x2 obj 7
 x2 r0 1
 x2 r1 3
 x2 r2 -4
 x2 r3 4
 x3 obj -7
 x3 r0 -2
 x3 r1 1
 x3 r3 -3
 x4 obj 5
 x4 r0 5
 x4 r1 -4
 x4 r2 4
 x4 r3 3
 x5 obj 0
 x5 r0 5
 x5 r1 2
 x5 r2 1
 x5 r3 -2
RHS
 rhs r0 -4
 rhs r1 6
 rhs r2 -1
 rhs r3 7
BOUNDS
 LI b x0 -3
 UI b x0 -1
 LI b x1 -2
 UI b x1 0
 LI b x2 0
 UI b x2 2
 LI b x3 -1
 UI b x3 0
 LI b x4 -3
 UI b x4 -1
 LO b x5 -2
 UP b x5 1
ENDATA
