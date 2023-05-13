* Simple RC circuit

V01 1 0 PULSE( 0.0 3.0 0.0 10p 10p 5n 10n )

R12 1 2 R=1000
C20 2 0 C=1p

.TRAN 40n 1n

.END
