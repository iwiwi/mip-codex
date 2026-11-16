NAME parity110
OBJSENSE
 MIN
ROWS
 N obj
 E r0
 E r1
 E r2
COLUMNS
 x0 obj 1.2
 x0 r0 3
 x0 r1 3
 x0 r2 -1
 x1 obj -1.3
 x1 r0 0
 x1 r1 0
 x1 r2 0
 x2 obj 1.7
 x2 r0 2
 x2 r1 1
 x2 r2 -1
 z0 obj 0.8 r0 -2
 z1 obj 0.7 r1 -2
 z2 obj 0.7 r2 -2
RHS
 b obj 0.1
 b r0 2
 b r1 1
 b r2 -1
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
