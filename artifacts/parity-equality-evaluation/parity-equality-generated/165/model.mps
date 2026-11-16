NAME parity165
OBJSENSE
 MAX
ROWS
 N obj
 E r0
COLUMNS
 x0 obj -1.0
 x0 r0 -2
 x1 obj -0.7
 x1 r0 2
 x2 obj -0.7
 x2 r0 -1
 x3 obj -1.8
 x3 r0 0
 x4 obj 0.1
 x4 r0 1
 z0 obj -0.6 r0 -2
RHS
 b obj -0.6
 b r0 -1
BOUNDS
 BV bd x0
 BV bd x1
 BV bd x2
 BV bd x3
 BV bd x4
 LI bd z0 -20
 UI bd z0 20
ENDATA
