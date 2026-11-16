NAME parity37
OBJSENSE
 MAX
ROWS
 N obj
 E r0
 E r1
COLUMNS
 x0 obj -0.2
 x0 r0 2
 x0 r1 2
 x1 obj 1.8
 x1 r0 2
 x1 r1 0
 x2 obj -1.9
 x2 r0 2
 x2 r1 1
 x3 obj -0.5
 x3 r0 -3
 x3 r1 -2
 x4 obj -2.0
 x4 r0 2
 x4 r1 2
 z0 obj 0.3 r0 -2
 z1 obj -0.9 r1 -2
RHS
 b obj -0.5
 b r0 1
 b r1 0
BOUNDS
 BV bd x0
 BV bd x1
 BV bd x2
 BV bd x3
 BV bd x4
 LI bd z0 -20
 UI bd z0 20
 LI bd z1 -20
 UI bd z1 20
ENDATA
