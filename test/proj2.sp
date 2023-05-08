* Proj 2

V01 1 0 2V
V03 3 0 3V

R12 1 2 R=450
R34 3 4 R=640
R45 4 5 R=450
R36 3 6 R=100

Q420 4 2 0 0 q_model
Q657 6 5 7 0 q_model

M720 7 2 0 0 t_model

.OP
.DC V01 0.0 3.0 10m

.END
