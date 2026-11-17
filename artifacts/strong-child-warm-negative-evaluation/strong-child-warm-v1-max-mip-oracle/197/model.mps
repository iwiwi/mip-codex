NAME gomory
OBJSENSE
 MAX
ROWS
 N obj
 L r0
 L r1
 L r2
COLUMNS
 x0 obj -3
 x0 r1 5
 x0 r2 5
 x1 obj -2
 x1 r0 -1
 x1 r1 4
 x1 r2 -5
 x2 obj -6
 x2 r0 -1
 x2 r1 3
 x2 r2 -1
 x3 obj 5
 x3 r2 4
 x4 obj -4
 x4 r0 3
 x4 r1 -3
RHS
 rhs r0 0
 rhs r1 -5
 rhs r2 16
BOUNDS
 LI b x0 -1
 UI b x0 0
 LO b x1 -3
 UP b x1 0
 LI b x2 -1
 UI b x2 0
 LI b x3 0
 UI b x3 3
 LI b x4 -3
 UI b x4 -1
ENDATA
