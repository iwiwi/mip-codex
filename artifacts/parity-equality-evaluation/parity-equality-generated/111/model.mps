NAME parity111
OBJSENSE
 MAX
ROWS
 N obj
 E r0
 E r1
 E r2
 E r3
 E r4
 E r5
COLUMNS
 x0 obj 0.5
 x0 r0 -2
 x0 r1 0
 x0 r2 -2
 x0 r3 -2
 x0 r4 3
 x0 r5 -1
 z0 obj -0.5 r0 -2
 z1 obj 1.7 r1 -2
 z2 obj 0.4 r2 -2
 z3 obj -0.8 r3 -2
 z4 obj 1.0 r4 -2
 z5 obj -0.5 r5 -2
RHS
 b obj -0.1
 b r0 -2
 b r1 0
 b r2 -2
 b r3 -2
 b r4 3
 b r5 -1
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
 LI bd z5 -20
 UI bd z5 20
ENDATA
