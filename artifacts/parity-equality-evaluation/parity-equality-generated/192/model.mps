NAME parity192
OBJSENSE
 MIN
ROWS
 N obj
 E r0
 E r1
 E r2
 E r3
COLUMNS
 x0 obj -1.2
 x0 r0 1
 x0 r1 -2
 x0 r2 -3
 x0 r3 0
 x1 obj 1.9
 x1 r0 3
 x1 r1 -2
 x1 r2 1
 x1 r3 -3
 x2 obj 0.2
 x2 r0 2
 x2 r1 3
 x2 r2 0
 x2 r3 1
 x3 obj -1.5
 x3 r0 -3
 x3 r1 1
 x3 r2 0
 x3 r3 -3
 z0 obj 0.0 r0 -2
 z1 obj 1.8 r1 -2
 z2 obj 1.3 r2 -2
 z3 obj 1.2 r3 -2
RHS
 b obj 0.9
 b r0 5
 b r1 -4
 b r2 -2
 b r3 -3
BOUNDS
 BV bd x0
 BV bd x1
 BV bd x2
 BV bd x3
 LI bd z0 -20
 UI bd z0 20
 LI bd z1 -20
 UI bd z1 20
 LI bd z2 -20
 UI bd z2 20
 LI bd z3 -20
 UI bd z3 20
ENDATA
