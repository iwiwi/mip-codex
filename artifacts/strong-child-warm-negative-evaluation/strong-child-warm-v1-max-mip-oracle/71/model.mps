NAME gomory
OBJSENSE
 MAX
ROWS
 N obj
 L r0
 L r1
COLUMNS
 x0 obj 7
 x0 r0 1
 x0 r1 4
 x1 obj -4
 x1 r0 -4
 x1 r1 -3
 x2 obj -2
 x2 r0 4
 x2 r1 1
 x3 obj -6
 x3 r1 3
 x4 obj 0
 x4 r0 -2
 x4 r1 -1
 x5 obj -2
 x5 r0 5
 x5 r1 -2
RHS
 rhs r0 -14
 rhs r1 -4
BOUNDS
 LI b x0 0
 UI b x0 2
 LI b x1 0
 UI b x1 3
 LI b x2 -2
 UI b x2 -1
 LI b x3 -3
 UI b x3 -1
 LI b x4 0
 UI b x4 3
 LO b x5 0
 UP b x5 3
ENDATA
