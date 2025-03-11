TEST=20 .bitor 10

.scope functions
.proc test

.endproc
.endscope

 test2:
    jmp #::functions::test