NAME parity168
OBJSENSE
 MIN
ROWS
 N obj
 E r0
 E r1
 E r2
 E r3
COLUMNS
 x0 obj 2.0
 x0 r0 -1
 x0 r1 0
 x0 r2 -1
 x0 r3 -2
 z0 obj 1.3 r0 -2
 z1 obj 1.2 r1 -2
 z2 obj 1.5 r2 -2
 z3 obj -1.2 r3 -2
RHS
 b obj -0.9
 b r0 0
 b r1 0
 b r2 -1
 b r3 -2
BOUNDS
 BV bd x0
 LI bd z0 -20
 UI bd z0 20
 LI bd z1 -20
 UI bd z1 20
 LI bd z2 -20
 UI bd z2 20
 LI bd z3 -20
 UI bd z3 20
ENDATA
