NAME gomory
OBJSENSE
 MAX
ROWS
 N obj
 L r0
 L r1
 L r2
COLUMNS
 x0 obj -6
 x0 r0 -3
 x1 obj -4
 x1 r0 1
 x1 r1 -5
 x1 r2 4
 x2 obj 2
 x2 r1 -5
 x2 r2 -2
RHS
 rhs r0 -3
 rhs r1 10
 rhs r2 5
BOUNDS
 LI b x0 0
 UI b x0 1
 LO b x1 -1
 UP b x1 0
 LI b x2 -2
 UI b x2 0
ENDATA
