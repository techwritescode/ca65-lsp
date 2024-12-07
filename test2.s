.include "test3.s"

PPU_STATUS = $8006

test1:
    lda $0
    ldx $09
@test:
    jsr test
@loop:
    sta $08
    lda PPU_STATUS
    jsr @loop
    jsr test1
    jsr test123
