NAME parity80
OBJSENSE
 MIN
ROWS
 N obj
 E r0
 E r1
COLUMNS
 x0 obj -1.4
 x0 r0 -1
 x0 r1 1
 x1 obj 0.3
 x1 r0 -2
 x1 r1 3
 x2 obj 1.4
 x2 r0 2
 x2 r1 -2
 x3 obj -0.9
 x3 r0 2
 x3 r1 -1
 x4 obj -2.0
 x4 r0 3
 x4 r1 0
 x5 obj 1.1
 x5 r0 1
 x5 r1 2
 z0 obj 1.2 r0 -2
 z1 obj -1.9 r1 -2
RHS
 b obj -0.7
 b r0 1
 b r1 4
BOUNDS
 BV bd x0
 BV bd x1
 BV bd x2
 BV bd x3
 BV bd x4
 BV bd x5
 LI bd z0 -20
 UI bd z0 20
 LI bd z1 -20
 UI bd z1 20
ENDATA
