NAME parity3
OBJSENSE
 MAX
ROWS
 N obj
 E r0
 E r1
 E r2
 E r3
COLUMNS
 x0 obj 0.0
 x0 r0 1
 x0 r1 3
 x0 r2 0
 x0 r3 -3
 x1 obj 0.9
 x1 r0 0
 x1 r1 -1
 x1 r2 3
 x1 r3 -2
 x2 obj 0.8
 x2 r0 -1
 x2 r1 -2
 x2 r2 3
 x2 r3 3
 x3 obj -0.9
 x3 r0 1
 x3 r1 1
 x3 r2 -1
 x3 r3 -1
 x4 obj -2.0
 x4 r0 3
 x4 r1 1
 x4 r2 2
 x4 r3 -1
 z0 obj -2.0 r0 -2
 z1 obj 0.7 r1 -2
 z2 obj 1.7 r2 -2
 z3 obj 1.4 r3 -2
RHS
 b obj -1.0
 b r0 0
 b r1 0
 b r2 6
 b r3 -2
BOUNDS
 BV bd x0
 BV bd x1
 BV bd x2
 BV bd x3
 BV bd x4
 LI bd z0 -20
 UI bd z0 20
 LI bd z1 -20
 UI bd z1 20
 LI bd z2 -20
 UI bd z2 20
 LI bd z3 -20
 UI bd z3 20
ENDATA
