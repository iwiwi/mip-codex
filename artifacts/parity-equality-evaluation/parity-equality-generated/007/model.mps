NAME parity7
OBJSENSE
 MAX
ROWS
 N obj
 E r0
 E r1
 E r2
COLUMNS
 x0 obj 1.3
 x0 r0 1
 x0 r1 3
 x0 r2 2
 x1 obj -2.0
 x1 r0 -2
 x1 r1 -1
 x1 r2 1
 z0 obj -1.3 r0 -2
 z1 obj -1.3 r1 -2
 z2 obj 1.6 r2 -2
RHS
 b obj 0.4
 b r0 -1
 b r1 2
 b r2 3
BOUNDS
 BV bd x0
 BV bd x1
 LI bd z0 -20
 UI bd z0 20
 LI bd z1 -20
 UI bd z1 20
 LI bd z2 -20
 UI bd z2 20
ENDATA
