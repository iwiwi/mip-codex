NAME parity4
OBJSENSE
 MIN
ROWS
 N obj
 E r0
 E r1
 E r2
 E r3
 E r4
COLUMNS
 x0 obj 1.1
 x0 r0 -1
 x0 r1 1
 x0 r2 1
 x0 r3 -2
 x0 r4 -3
 z0 obj 1.9 r0 -2
 z1 obj -1.9 r1 -2
 z2 obj -1.6 r2 -2
 z3 obj 0.2 r3 -2
 z4 obj 0.9 r4 -2
RHS
 b obj 1.0
 b r0 0
 b r1 1
 b r2 1
 b r3 -2
 b r4 -3
BOUNDS
 BV bd x0
 LI bd z0 -20
 UI bd z0 20
 LI bd z1 -20
 UI bd z1 20
 LI bd z2 -20
 UI bd z2 20
 LI bd z3 -20
 UI bd z3 20
 LI bd z4 -20
 UI bd z4 20
ENDATA
