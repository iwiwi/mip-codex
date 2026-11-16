NAME parity141
OBJSENSE
 MAX
ROWS
 N obj
 E r0
 E r1
COLUMNS
 x0 obj -0.3
 x0 r0 1
 x0 r1 3
 x1 obj -1.9
 x1 r0 1
 x1 r1 1
 z0 obj -1.8 r0 -2
 z1 obj 1.0 r1 -2
RHS
 b obj 0.9
 b r0 2
 b r1 4
BOUNDS
 BV bd x0
 BV bd x1
 LI bd z0 -20
 UI bd z0 20
 LI bd z1 -20
 UI bd z1 20
ENDATA
