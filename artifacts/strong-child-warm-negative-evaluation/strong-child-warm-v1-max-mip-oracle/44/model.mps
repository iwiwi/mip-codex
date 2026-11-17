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
 x0 r0 1
 x0 r1 -4
 x0 r2 -3
 x0 r3 5
 x1 obj -4
 x1 r0 5
 x1 r1 4
 x1 r2 1
 x1 r3 -2
 x2 obj 0
 x2 r0 -1
 x2 r1 -4
 x2 r2 5
 x2 r3 -1
 x3 obj -1
 x3 r0 2
 x3 r1 1
 x3 r2 -5
 x3 r3 -5
RHS
 rhs r0 9
 rhs r1 7
 rhs r2 -3
 rhs r3 8
BOUNDS
 LI b x0 0
 UI b x0 1
 LO b x1 1
 UP b x1 4
 LO b x2 -1
 UP b x2 2
 LI b x3 0
 UI b x3 3
ENDATA
