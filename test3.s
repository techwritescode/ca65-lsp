MY_CONST = $10

.proc test_proc

.endproc

.macro test param
    jmp param
.endmacro

main:
    jmp main
    test 1, 2, 3
    jmp test_proc
    lda #MY_CONST