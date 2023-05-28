* Proj 3

V01 1 0 EXP(3.0 0.0 0.0 2n 40n 2n)
V08 8 0 3V

R12 1 2 R=450
R34 3 4 R=640
R45 4 5 R=450
R36 3 6 R=100

Q420 4 2 0 0 q_model
Q657 6 5 7 0 q_model

M720 7 2 0 0 t_model

C20 2 0 C=2p
C50 5 0 C=2p
C70 7 0 C=10p

L83 8 3 L=0.3u

*.OP
.TRAN 40n 5n

.END
