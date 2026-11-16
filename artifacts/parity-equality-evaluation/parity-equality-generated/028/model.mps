NAME parity28
OBJSENSE
 MIN
ROWS
 N obj
 E r0
 E r1
 E r2
 E r3
COLUMNS
 x0 obj -0.4
 x0 r0 3
 x0 r1 -3
 x0 r2 -1
 x0 r3 -2
 x1 obj -1.2
 x1 r0 -2
 x1 r1 1
 x1 r2 1
 x1 r3 2
 z0 obj 0.8 r0 -2
 z1 obj -1.7 r1 -2
 z2 obj 1.7 r2 -2
 z3 obj -0.7 r3 -2
RHS
 b obj -1.0
 b r0 -1
 b r1 1
 b r2 1
 b r3 2
BOUNDS
 BV bd x0
 BV bd x1
 LI bd z0 -20
 UI bd z0 20
 LI bd z1 -20
 UI bd z1 20
 LI bd z2 -20
 UI bd z2 20
 LI bd z3 -20
 UI bd z3 20
ENDATA
