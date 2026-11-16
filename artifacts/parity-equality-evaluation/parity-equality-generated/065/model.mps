NAME parity65
OBJSENSE
 MAX
ROWS
 N obj
 E r0
 E r1
COLUMNS
 x0 obj -2.0
 x0 r0 -1
 x0 r1 1
 x1 obj 0.5
 x1 r0 2
 x1 r1 0
 z0 obj 0.8 r0 -2
 z1 obj 1.2 r1 -2
RHS
 b obj 1.0
 b r0 2
 b r1 0
BOUNDS
 BV bd x0
 BV bd x1
 LI bd z0 -20
 UI bd z0 20
 LI bd z1 -20
 UI bd z1 20
ENDATA
