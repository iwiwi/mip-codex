NAME parity126
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
 x0 r0 1
 x0 r1 2
 x0 r2 3
 x0 r3 3
 x1 obj 1.9
 x1 r0 -3
 x1 r1 3
 x1 r2 -1
 x1 r3 -3
 z0 obj -0.2 r0 -2
 z1 obj 0.9 r1 -2
 z2 obj -1.0 r2 -2
 z3 obj -0.6 r3 -2
RHS
 b obj 0.2
 b r0 -2
 b r1 5
 b r2 2
 b r3 0
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
