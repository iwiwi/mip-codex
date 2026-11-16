NAME parity8
OBJSENSE
 MIN
ROWS
 N obj
 E r0
 E r1
 E r2
 E r3
 E r4
 E r5
COLUMNS
 x0 obj 0.4
 x0 r0 3
 x0 r1 3
 x0 r2 -3
 x0 r3 -3
 x0 r4 -1
 x0 r5 -2
 x1 obj -1.9
 x1 r0 1
 x1 r1 -2
 x1 r2 -2
 x1 r3 -1
 x1 r4 -2
 x1 r5 -1
 z0 obj 0.6 r0 -2
 z1 obj -1.1 r1 -2
 z2 obj 0.4 r2 -2
 z3 obj -1.8 r3 -2
 z4 obj -1.7 r4 -2
 z5 obj -0.9 r5 -2
RHS
 b obj -0.3
 b r0 1
 b r1 0
 b r2 0
 b r3 0
 b r4 0
 b r5 0
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
 LI bd z4 -20
 UI bd z4 20
 LI bd z5 -20
 UI bd z5 20
ENDATA
