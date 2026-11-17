NAME gomory
OBJSENSE
 MAX
ROWS
 N obj
 L r0
 L r1
COLUMNS
 x0 obj -4
 x0 r0 1
 x1 obj 7
 x1 r0 3
 x1 r1 5
 x2 obj -6
 x2 r0 1
 x2 r1 -5
 x3 obj -6
 x3 r0 -1
 x3 r1 3
 x4 obj -2
 x4 r0 -4
 x4 r1 3
RHS
 rhs r0 5
 rhs r1 -13
BOUNDS
 LI b x0 0
 UI b x0 1
 LO b x1 -2
 UP b x1 -1
 LI b x2 -1
 UI b x2 2
 LI b x3 0
 UI b x3 2
 LO b x4 -2
 UP b x4 0
ENDATA
