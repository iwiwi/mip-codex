NAME parity98
OBJSENSE
 MIN
ROWS
 N obj
 E r0
 E r1
COLUMNS
 x0 obj -1.2
 x0 r0 -3
 x0 r1 -3
 x1 obj -1.6
 x1 r0 2
 x1 r1 -2
 x2 obj 2.0
 x2 r0 1
 x2 r1 1
 x3 obj -1.8
 x3 r0 3
 x3 r1 0
 z0 obj 0.0 r0 -2
 z1 obj 0.0 r1 -2
RHS
 b obj -0.5
 b r0 -1
 b r1 -5
BOUNDS
 BV bd x0
 BV bd x1
 BV bd x2
 BV bd x3
 LI bd z0 -20
 UI bd z0 20
 LI bd z1 -20
 UI bd z1 20
ENDATA
