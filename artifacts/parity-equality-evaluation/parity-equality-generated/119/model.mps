NAME parity119
OBJSENSE
 MAX
ROWS
 N obj
 E r0
 E r1
 E r2
 E r3
 E r4
COLUMNS
 x0 obj 1.1
 x0 r0 3
 x0 r1 1
 x0 r2 -1
 x0 r3 1
 x0 r4 0
 x1 obj 1.1
 x1 r0 -1
 x1 r1 0
 x1 r2 1
 x1 r3 -2
 x1 r4 3
 x2 obj -1.5
 x2 r0 -2
 x2 r1 1
 x2 r2 -2
 x2 r3 -2
 x2 r4 1
 z0 obj -1.6 r0 -2
 z1 obj -0.1 r1 -2
 z2 obj 0.5 r2 -2
 z3 obj -0.6 r3 -2
 z4 obj 0.1 r4 -2
RHS
 b obj 0.2
 b r0 -1
 b r1 0
 b r2 1
 b r3 -2
 b r4 3
BOUNDS
 BV bd x0
 BV bd x1
 BV bd x2
 LI bd z0 -20
 UI bd z0 20
 LI bd z1 -20
 UI bd z1 20
 LI bd z2 -20
 UI bd z2 20
 LI bd z3 -20
 UI bd z3 20
 LI bd z4 -20
 UI bd z4 20
ENDATA
