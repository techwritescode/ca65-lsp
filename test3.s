MY_CONST = $10
ScreenEdge_X_Pos = $20

.proc test_proc

.endproc

.macro test param
    jmp param
.endmacro

main:
    jmp main
    test 1, 2, 3
    jmp test_proc
    jmp t
    lda #MY_CONST
    lda #ScreenEdge_X_Pos