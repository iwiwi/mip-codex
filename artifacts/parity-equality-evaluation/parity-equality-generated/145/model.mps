NAME parity145
OBJSENSE
 MAX
ROWS
 N obj
 E r0
 E r1
 E r2
COLUMNS
 x0 obj 1.7
 x0 r0 1
 x0 r1 1
 x0 r2 2
 x1 obj 1.4
 x1 r0 1
 x1 r1 -1
 x1 r2 -1
 x2 obj 0.1
 x2 r0 1
 x2 r1 0
 x2 r2 -2
 z0 obj 0.6 r0 -2
 z1 obj 1.0 r1 -2
 z2 obj 1.9 r2 -2
RHS
 b obj -0.9
 b r0 2
 b r1 1
 b r2 0
BOUNDS
 BV bd x0
 BV bd x1
 BV bd x2
 LI bd z0 -20
 UI bd z0 20
 LI bd z1 -20
 UI bd z1 20
 LI bd z2 -20
 UI bd z2 20
ENDATA
