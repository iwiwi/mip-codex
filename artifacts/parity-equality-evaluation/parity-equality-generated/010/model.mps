NAME parity10
OBJSENSE
 MIN
ROWS
 N obj
 E r0
 E r1
 E r2
 E r3
COLUMNS
 x0 obj 0.7
 x0 r0 2
 x0 r1 -1
 x0 r2 -2
 x0 r3 3
 z0 obj 0.1 r0 -2
 z1 obj 0.5 r1 -2
 z2 obj 0.5 r2 -2
 z3 obj 1.6 r3 -2
RHS
 b obj -0.9
 b r0 2
 b r1 -1
 b r2 -2
 b r3 3
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
ENDATA
