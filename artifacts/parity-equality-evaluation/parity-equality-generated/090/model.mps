NAME parity90
OBJSENSE
 MIN
ROWS
 N obj
 E r0
 E r1
 E r2
COLUMNS
 x0 obj 0.0
 x0 r0 -2
 x0 r1 1
 x0 r2 -2
 z0 obj -1.7 r0 -2
 z1 obj -0.3 r1 -2
 z2 obj 0.0 r2 -2
RHS
 b obj -0.6
 b r0 0
 b r1 0
 b r2 0
BOUNDS
 BV bd x0
 LI bd z0 -20
 UI bd z0 20
 LI bd z1 -20
 UI bd z1 20
 LI bd z2 -20
 UI bd z2 20
ENDATA
