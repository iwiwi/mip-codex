NAME parity97
OBJSENSE
 MAX
ROWS
 N obj
 E r0
 E r1
 E r2
COLUMNS
 x0 obj 1.9
 x0 r0 1
 x0 r1 -1
 x0 r2 3
 z0 obj 1.6 r0 -2
 z1 obj 1.1 r1 -2
 z2 obj 0.1 r2 -2
RHS
 b obj -0.9
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
