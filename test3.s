TEST=20 .bitor 10

.scope functions
    .proc test
        lda #TEST
    .endproc
.endscope

 test2:
    jmp #::functions::test
    lda #0
    add (10 * 20 - 5)@@