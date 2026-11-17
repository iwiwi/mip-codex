NAME gomory
OBJSENSE
 MAX
ROWS
 N obj
 L r0
 L r1
 L r2
COLUMNS
 x0 obj -5
 x0 r0 -4
 x0 r1 -2
 x0 r2 1
 x1 obj -1
 x1 r0 -1
 x1 r1 -2
 x1 r2 2
 x2 obj 4
 x2 r0 -3
 x2 r2 -3
RHS
 rhs r0 -1
 rhs r1 3
 rhs r2 2
BOUNDS
 LI b x0 -1
 UI b x0 2
 LO b x1 -2
 UP b x1 -1
 LO b x2 -1
 UP b x2 2
ENDATA
