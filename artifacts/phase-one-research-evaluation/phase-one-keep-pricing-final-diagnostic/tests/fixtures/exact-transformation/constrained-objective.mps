NAME constrained-cancel
ROWS
 N c
 E rx
 E ry
 E rz
COLUMNS
 x c 10000000000000000 rx 1
 y c 1 ry 1
 z c -10000000000000000 rz 1
RHS
 rhs rx 1 ry 1
 rhs rz 1
BOUNDS
 UP b x 2
 UP b y 2
 UP b z 2
ENDATA
