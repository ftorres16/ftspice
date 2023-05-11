* Simple RC circuit

V01 1 0 SIN( 0.0 3.0 100M )

R12 1 2 R=1000
C20 2 0 C=10p

.TRAN 0 40n 1n

.END
