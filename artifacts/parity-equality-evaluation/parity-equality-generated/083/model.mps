NAME parity83
OBJSENSE
 MAX
ROWS
 N obj
 E r0
 E r1
COLUMNS
 x0 obj -1.4
 x0 r0 -3
 x0 r1 1
 x1 obj -0.1
 x1 r0 -3
 x1 r1 -3
 x2 obj -0.1
 x2 r0 3
 x2 r1 -1
 x3 obj -2.0
 x3 r0 3
 x3 r1 -2
 z0 obj 0.3 r0 -2
 z1 obj -2.0 r1 -2
RHS
 b obj -1.0
 b r0 0
 b r1 -5
BOUNDS
 BV bd x0
 BV bd x1
 BV bd x2
 BV bd x3
 LI bd z0 -20
 UI bd z0 20
 LI bd z1 -20
 UI bd z1 20
ENDATA
