NAME parity153
OBJSENSE
 MAX
ROWS
 N obj
 E r0
 E r1
COLUMNS
 x0 obj 1.8
 x0 r0 3
 x0 r1 3
 x1 obj -0.7
 x1 r0 0
 x1 r1 1
 x2 obj 0.5
 x2 r0 -1
 x2 r1 2
 z0 obj -0.1 r0 -2
 z1 obj -1.2 r1 -2
RHS
 b obj 0.6
 b r0 3
 b r1 4
BOUNDS
 BV bd x0
 BV bd x1
 BV bd x2
 LI bd z0 -20
 UI bd z0 20
 LI bd z1 -20
 UI bd z1 20
ENDATA
