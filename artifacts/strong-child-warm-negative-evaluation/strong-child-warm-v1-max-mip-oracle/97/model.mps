NAME gomory
OBJSENSE
 MAX
ROWS
 N obj
 L r0
 L r1
COLUMNS
 x0 obj -2
 x0 r0 -4
 x0 r1 -4
 x1 obj -1
 x1 r0 4
 x1 r1 2
 x2 obj -2
 x2 r0 -1
 x2 r1 1
 x3 obj -1
 x3 r0 -5
 x3 r1 -1
RHS
 rhs r0 31
 rhs r1 19
BOUNDS
 LI b x0 -3
 UI b x0 0
 LI b x1 1
 UI b x1 3
 LI b x2 -1
 UI b x2 1
 LO b x3 -1
 UP b x3 0
ENDATA
