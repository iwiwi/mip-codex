NAME parity1
OBJSENSE
 MAX
ROWS
 N obj
 E r0
 E r1
 E r2
 E r3
COLUMNS
 x0 obj -0.9
 x0 r0 -3
 x0 r1 -3
 x0 r2 -2
 x0 r3 -3
 x1 obj -1.2
 x1 r0 -1
 x1 r1 -3
 x1 r2 2
 x1 r3 -1
 x2 obj -1.4
 x2 r0 -1
 x2 r1 0
 x2 r2 1
 x2 r3 -1
 z0 obj 0.8 r0 -2
 z1 obj 2.0 r1 -2
 z2 obj 1.9 r2 -2
 z3 obj 0.8 r3 -2
RHS
 b obj 0.0
 b r0 -4
 b r1 -3
 b r2 -1
 b r3 -4
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
 LI bd z3 -20
 UI bd z3 20
ENDATA
