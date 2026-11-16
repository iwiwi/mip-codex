NAME toy
ROWS
 N obj
 G demand
 L cap
COLUMNS
 x obj -1 demand 1
 x cap 3
 y demand 1 cap -1
RHS
 rhs demand 2 cap 2
BOUNDS
 LI b x 0
 UP b x 3
 LO b y 0
 UP b y 3
ENDATA
