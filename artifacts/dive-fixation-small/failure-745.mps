NAME synthetic
OBJSENSE
 MAX
ROWS
 N obj
 G r0
 E r1
 E r2
COLUMNS
 x0 obj 1
 x0 r0 2
 x0 r2 -2
 x1 obj -5
 x1 r0 3
 x1 r1 -1
 x1 r2 1
 x2 obj -1
 x2 r0 3
 x2 r1 -3
 x2 r2 1
 x3 obj 0
 x3 r0 2
 x3 r1 1
 x3 r2 1
 x4 obj 5
 x4 r0 3
 x4 r1 -2
 x4 r2 -3
RHS
 rhs obj 0
 rhs r0 -4
 rhs r1 -4
 rhs r2 1
BOUNDS
 LI bounds x0 0
 UP bounds x0 1
 LI bounds x1 -3
 UP bounds x1 2
 LI bounds x2 -1
 UP bounds x2 1
 LO bounds x3 0
 UP bounds x3 4
 LO bounds x4 -1
 UP bounds x4 3
ENDATA
