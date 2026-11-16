NAME parity33
OBJSENSE
 MAX
ROWS
 N obj
 E r0
 E r1
COLUMNS
 x0 obj -0.1
 x0 r0 -2
 x0 r1 -3
 z0 obj -1.2 r0 -2
 z1 obj 1.2 r1 -2
RHS
 b obj -0.6
 b r0 0
 b r1 0
BOUNDS
 BV bd x0
 LI bd z0 -20
 UI bd z0 20
 LI bd z1 -20
 UI bd z1 20
ENDATA
