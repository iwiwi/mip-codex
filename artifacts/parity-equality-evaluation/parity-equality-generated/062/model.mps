NAME parity62
OBJSENSE
 MIN
ROWS
 N obj
 E r0
 E r1
COLUMNS
 x0 obj 1.9
 x0 r0 2
 x0 r1 1
 z0 obj -1.2 r0 -2
 z1 obj 0.4 r1 -2
RHS
 b obj -0.8
 b r0 2
 b r1 1
BOUNDS
 BV bd x0
 LI bd z0 -20
 UI bd z0 20
 LI bd z1 -20
 UI bd z1 20
ENDATA
