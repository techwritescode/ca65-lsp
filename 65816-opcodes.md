ADC - Add with Carry
====================

**Flags affected**: `nv----zc`

`A` ← `A + M + c`

`n` ← Most significant bit of result

`v` ← Signed overflow of result

`z` ← Set if the result is zero

`c` ← Carry from ALU (bit 8/16 of result)
---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`ADC #const`      | Immediate                 | `$69`   | 2 / 3 | 2 |
`ADC addr`        | Absolute                  | `$6D`   | 3     | 4 |
`ADC long`        | Absolute Long             | `$6F`   | 4     | 5 |
`ADC dp`          | Direct Page               | `$65`   | 2     | 3 |
`ADC (dp)`        | Direct Page Indirect      | `$72`   | 2     | 5 |
`ADC [dp]`        | Direct Page Indirect Long | `$67`   | 2     | 6 |
`ADC addr, X`     | Absolute Indexed, X       | `$7D`   | 3     | 4 |
`ADC long, X`     | Absolute Long Indexed, X  | `$7F`   | 4     | 5 |
`ADC addr, Y`     | Absolute Indexed, Y       | `$79`   | 3     | 4 |
`ADC dp, X`       | Direct Page Indexed, X    | `$75`   | 2     | 4 |
`ADC (dp, X)`     | Direct Page Indirect, X   | `$61`   | 2     | 6 |
`ADC (dp), Y`     | DP Indirect Indexed, Y    | `$71`   | 2     | 5 |
`ADC [dp], Y`     | DP Indirect Long Indexed, Y | `$77` | 2     | 6 |
`ADC sr, S`       | Stack Relative            | `$63`   | 2     | 4 |
`ADC (sr, S), Y`  | SR Indirect Indexed, Y    | `$73`   | 2     | 7 |



AND - And Accumulator with Memory
=================================

**Flags affected**: `n-----z-`

`A` ← `A & M`

---
`n` ← Most significant bit of result

`z` ← Set if the result is zero

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`AND #const`      | Immediate                 | `$29`   | 2 / 3 | 2 |
`AND addr`        | Absolute                  | `$2D`   | 3     | 4 |
`AND long`        | Absolute Long             | `$2F`   | 4     | 5 |
`AND dp`          | Direct Page               | `$25`   | 2     | 3 |
`AND (dp)`        | Direct Page Indirect      | `$32`   | 2     | 5 |
`AND [dp]`        | Direct Page Indirect Long | `$27`   | 2     | 6 |
`AND addr, X`     | Absolute Indexed, X       | `$3D`   | 3     | 4 |
`AND long, X`     | Absolute Long Indexed, X  | `$3F`   | 4     | 5 |
`AND addr, Y`     | Absolute Indexed, Y       | `$39`   | 3     | 4 |
`AND dp, X`       | Direct Page Indexed, X    | `$35`   | 2     | 4 |
`AND (dp, X)`     | Direct Page Indirect, X   | `$21`   | 2     | 6 |
`AND (dp), Y`     | DP Indirect Indexed, Y    | `$31`   | 2     | 5 |
`AND [dp], Y`     | DP Indirect Long Indexed, Y | `$37` | 2     | 6 |
`AND sr, S`       | Stack Relative            | `$23`   | 2     | 4 |
`AND (sr, S), Y`  | SR Indirect Indexed, Y    | `$33`   | 2     | 7 |



ASL - Arithmetic Shift Left
===========================

**Flags affected**: `n-----zc`

`M` ← `M + M`

---
`n` ← Most significant bit of result

`z` ← Set if the result is zero

`c` ← Most significant bit of original Memory

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`ASL`             | Accumulator               | `$0A`   | 1     | 2
`ASL addr`        | Absolute                  | `$0E`   | 3     | 6 |
`ASL dp`          | Direct Page               | `$06`   | 2     | 5 |
`ASL addr, X`     | Absolute Indexed, X       | `$1E`   | 3     | 7 |
`ASL dp, X`       | Direct Page Indexed, X    | `$16`   | 2     | 6 |



Branches
========

**Flags affected**: `--------`

**Branch not taken:**

&mdash;

**Branch taken:**

`PC` ← `PC + sign-extend(near)`

**Branch taken (BRL):**

`PC` ← `PC + label`

---
Syntax          | Name                      | Condition       |     | Opcode| Bytes | Cycles |
----------------|---------------------------|-----------------|-----|-------|-------|--------|
`BCC near`        | Branch if Carry Clear     | carry clear     | c=0 | `$90`   | 2     | 2 |
`BCS near`        | Branch if Carry Set       | carry set       | c=1 | `$B0`   | 2     | 2 |
`BNE near`        | Branch if Not Equal       | zero clear      | z=0 | `$D0`   | 2     | 2 |
`BEQ near`        | Branch if Equal           | zero set        | z=1 | `$F0`   | 2     | 2 |
`BPL near`        | Branch if Plus            | negative clear  | n=0 | `$10`   | 2     | 2 |
`BMI near`        | Branch if Minus           | negative set    | n=1 | `$30`   | 2     | 2 |
`BVC near`        | Branch if Overflow Clear  | overflow clear  | v=0 | `$50`   | 2     | 2 |
`BVS near`        | Branch if Overflow Set    | overflow set    | v=1 | `$70`   | 2     | 2 |
`BRA near`        | Branch Always             | always          |     | `$80`   | 2     | 3 |
`BRL label`       | Branch Always Long        | always          |     | `$82`   | 3     | 4



BIT - Test Memory Bits against Accumulator
==========================================

**Flags affected**: `nv----z-`

**Flags affected (Immediate addressing mode only)**: `------z-`

`A & M`

---
`n` ← Most significant bit of memory

`v` ← Second most significant bit of memory

`z` ← Set if logical AND of memory and Accumulator is zero


---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`BIT #const`      | Immediate                 | `$89`   | 2 / 3 | 2 |
`BIT addr`        | Absolute                  | `$2C`   | 3     | 4 |
`BIT dp`          | Direct Page               | `$24`   | 2     | 3 |
`BIT addr, X`     | Absolute Indexed, X       | `$3C`   | 3     | 4 |
`BIT dp, X`       | Direct Page Indexed, X    | `$34`   | 2     | 4 |



Software Interrupts
===================

**Flags affected**: `----di--`

**Native Mode:**
`S` ← `S - 4`
`[S+4]` ← `PBR`
`[S+3]` ← `PC.h`
`[S+2]` ← `PC.l`
`[S+1]` ← `P`


`d` ← `0`
`i` ← `1`


`PBR` ← `0`
`PC` ← interrupt address


**Emulation Mode:**
`S` ← `S - 3`
`[S+3]` ← `PC.h`
`[S+2]` ← `PC.l`
`[S+1]` ← `P`


`d` ← `0`
`i` ← `1`


`PBR` ← `0`
`PC` ← interrupt address

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`BRK param`       | Interrupt                 | `$00`   | 2     | 7 |
`COP param`       | Interrupt                 | `$02`   | 2     | 7 |



Clear Status Flags
==================

**Flags affected (`CLC`)**: `-------c`

**Flags affected (`CLD`)**: `----d---`

**Flags affected (`CLI`)**: `-----i--`

**Flags affected (`CLV`)**: `-v------`

**CLC:**

`c` ← `0`

**CLD:**

`d` ← `0`

**CLI:**

`i` ← `0`

**CLV:**

`v` ← `0`
---
Syntax          | Name                      | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`CLC`             | Clear Carry Flag          | `$18`   | 1     | 2
`CLI`             | Clear Interrupt Disable Flag | `$58`| 1     | 2
`CLD`             | Clear Decimal Flag        | `$D8`   | 1     | 2
`CLV`             | Clear Overflow Flag       | `$B8`   | 1     | 2



CMP - Compare Accumulator with Memory
=====================================

**Flags affected**: `n-----zc`

`A - M`

---
`n` ← Most significant bit of result

`z` ← Set if the result is zero (Set if A == M)

`c` ← Carry from ALU (Set if A >= M)

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`CMP #const`      | Immediate                 | `$C9`   | 2 / 3 | 2 |
`CMP addr`        | Absolute                  | `$CD`   | 3     | 4 |
`CMP long`        | Absolute Long             | `$CF`   | 4     | 5 |
`CMP dp`          | Direct Page               | `$C5`   | 2     | 3 |
`CMP (dp)`        | Direct Page Indirect      | `$D2`   | 2     | 5 |
`CMP [dp]`        | Direct Page Indirect Long | `$C7`   | 2     | 6 |
`CMP addr, X`     | Absolute Indexed, X       | `$DD`   | 3     | 4 |
`CMP long, X`     | Absolute Long Indexed, X  | `$DF`   | 4     | 5 |
`CMP addr, Y`     | Absolute Indexed, Y       | `$D9`   | 3     | 4 |
`CMP dp, X`       | Direct Page Indexed, X    | `$D5`   | 2     | 4 |
`CMP (dp, X)`     | Direct Page Indirect, X   | `$C1`   | 2     | 6 |
`CMP (dp), Y`     | DP Indirect Indexed, Y    | `$D1`   | 2     | 5 |
`CMP [dp], Y`     | DP Indirect Long Indexed, Y | `$D7` | 2     | 6 |
`CMP sr, S`       | Stack Relative            | `$C3`   | 2     | 4 |
`CMP (sr, S), Y`  | SR Indirect Indexed, Y    | `$D3`   | 2     | 7 |



CPX - Compare Index Register X with Memory
==========================================

**Flags affected**: `n-----zc`

`X - M`

---
`n` ← Most significant bit of result

`z` ← Set if the result is zero (Set if `X == M`)

`c` ← Carry from ALU (Set if `X >= M`)

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`CPX #const`      | Immediate                 | `$E0`   | 2 / 3 | 2 |
`CPX addr`        | Absolute                  | `$EC`   | 3     | 4 |
`CPX dp`          | Direct Page               | `$E4`   | 2     | 3 |



CPY - Compare Index Register Y with Memory
==========================================

**Flags affected**: `n-----zc`

`Y - M`

---
`n` ← Most significant bit of result

`z` ← Set if the result is zero (Set if `Y == M`)

`c` ← Carry from ALU (Set if `Y >= M`)

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`CPY #const`      | Immediate                 | `$C0`   | 2 / 3 | 2 |
`CPY addr`        | Absolute                  | `$CC`   | 3     | 4 |
`CPY dp`          | Direct Page               | `$C4`   | 2     | 3 |



DEC - Decrement
===============

**Flags affected**: `n-----z-`

`M` ← `M - 1`

---
`n` ← Most significant bit of result

`z` ← Set if the result is zero

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`DEC`             | Accumulator               | `$3A`   | 1     | 2
`DEC addr`        | Absolute                  | `$CE`   | 3     | 6 |
`DEC dp`          | Direct Page               | `$C6`   | 2     | 5 |
`DEC addr, X`     | Absolute Indexed, X       | `$DE`   | 3     | 7 |
`DEC dp, X`       | Direct Page Indexed, X    | `$D6`   | 2     | 6 |



DEX, DEY - Decrement Index Registers
====================================

**Flags affected**: `n-----z-`

`R` ← `R - 1`

---
`n` ← Most significant bit of result

`z` ← Set if the result is zero

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`DEX`             | Implied                   | `$CA`   | 1     | 2
`DEY`             | Implied                   | `$88`   | 1     | 2



EOR - Exclusive OR Accumulator with Memory
==========================================

**Flags affected**: `n-----z-`

`A` ← `A ^ M`

---
`n` ← Most significant bit of result

`z` ← Set if the result is zero

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`EOR #const`      | Immediate                 | `$49`   | 2 / 3 | 2 |
`EOR addr`        | Absolute                  | `$4D`   | 3     | 4 |
`EOR long`        | Absolute Long             | `$4F`   | 4     | 5 |
`EOR dp`          | Direct Page               | `$45`   | 2     | 3 |
`EOR (dp)`        | Direct Page Indirect      | `$52`   | 2     | 5 |
`EOR [dp]`        | Direct Page Indirect Long | `$47`   | 2     | 6 |
`EOR addr, X`     | Absolute Indexed, X       | `$5D`   | 3     | 4 |
`EOR long, X`     | Absolute Long Indexed, X  | `$5F`   | 4     | 5 |
`EOR addr, Y`     | Absolute Indexed, Y       | `$59`   | 3     | 4 |
`EOR dp, X`       | Direct Page Indexed, X    | `$55`   | 2     | 4 |
`EOR (dp, X)`     | Direct Page Indirect, X   | `$41`   | 2     | 6 |
`EOR (dp), Y`     | DP Indirect Indexed, Y    | `$51`   | 2     | 5 |
`EOR [dp], Y`     | DP Indirect Long Indexed, Y | `$57` | 2     | 6 |
`EOR sr, S`       | Stack Relative            | `$43`   | 2     | 4 |
`EOR (sr, S), Y`  | SR Indirect Indexed, Y    | `$53`   | 2     | 7 |



INC - Increment
===============

**Flags affected**: `n-----z-`

`M` ← `M + 1`

---
`n` ← Most significant bit of result

`z` ← Set if the result is zero

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`INC`             | Accumulator               | `$1A`   | 1     | 2
`INC addr`        | Absolute                  | `$EE`   | 3     | 6 |
`INC dp`          | Direct Page               | `$E6`   | 2     | 5 |
`INC addr, X`     | Absolute Indexed, X       | `$FE`   | 3     | 7 |
`INC dp, X`       | Direct Page Indexed, X    | `$F6`   | 2     | 6 |



INX, INY - Increment Index Registers
====================================

**Flags affected**: `n-----z-`

`R` ← `R + 1`

---
`n` ← Most significant bit of result

`z` ← Set if the result is zero

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`INX`             | Implied                   | `$E8`   | 1     | 2
`INY`             | Implied                   | `$C8`   | 1     | 2



JMP, JML - Jump
===============

**Flags affected**: `--------`

**JMP:**
`PC` ← `M`

**JML:**
`PBR:PC` ← `M`




NOTES:

 * The `JMP (addr)` instruction will always read the new program counter from Bank 0 (ie, `JMP ($8888)` will read 2 bytes from `$00:8888`).
 * The `JML [addr]` instruction will always read the new program counter from Bank 0 (ie, `JML [$9999]` will read 3 bytes from `$00:9999`).
 * The `JMP (addr, X)` instruction will read the new program counter from the Program Bank (`PBR`) (ie, `JMP ($AAAA, X)` will read 2 bytes from `PBR:{$AAAA + X}`).


<table>
<thead>
  <tr>              <th>Syntax</th>         <th>Addressing Mode</th>                <th>Opcode</th> <th>Bytes</th> <th>Cycles</th> <th>Extra</th> </tr>
</thead>
<tbody>
  <tr class="odd">  <td>JMP addr</td>       <td>Absolute</td>                       <td>$4C</td> <td>3</td> <td>3</td> <td></td> </tr>
  <tr class="even"> <td>JMP (addr)</td>     <td>Absolute Indirect</td>              <td>$6C</td> <td>3</td> <td>5</td> <td></td> </tr>
  <tr class="odd">  <td>JMP (addr, X)</td>  <td>Absolute Indexed Indirect, X</td>   <td>$7C</td> <td>3</td> <td>6</td> <td></td> </tr>

  <tr class="even"> <td>JML long</td>       <td rowspan="2">Absolute Long</td>          <td rowspan="2">$5C</td> <td rowspan="2">4</td> <td rowspan="2">4</td> <td rowspan="2"></td> </tr>
  <tr class="even"> <td>JMP long</td> </tr>
  <tr class="odd">  <td>JML [addr]</td>     <td rowspan="2">Absolute Indirect Long</td> <td rowspan="2">$DC</td> <td rowspan="2">3</td> <td rowspan="2">6</td> <td rowspan="2"></td> </tr>
  <tr class="odd">  <td>JMP [addr]</td> </tr>
</tbody>
</table>



JSR, JSL - Jump to Subroutine
=============================

**Flags affected**: `--------`

**JSR:**
`PC` ← `PC - 1`
`S` ← `S - 2`
`[S+2]` ← `PC.h`
`[S+1]` ← `PC.l`
`PC` ← `M`

**JSL:**
`PC` ← `PC - 1`
`S` ← `S - 3`
`[S+3]` ← `PBR`
`[S+2]` ← `PC.h`
`[S+1]` ← `PC.l`
`PBR:PC` ← `M`

NOTE: The `JSR (addr, X)` instruction will read the subroutine address from the Program Bank (`PBR`) (ie, `JSR {$8888, X}` will read 2 bytes from `PBR:{$8888 + X}`).

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`JSR addr`        | Absolute                  | `$20`   | 3     | 6
`JSR (addr, X)`   | Absolute Indexed Indirect, X | `$FC`| 3     | 8
`JSL long`        | Absolute Long             | `$22`   | 4     | 8



LDA - Load Accumulator from Memory
==================================

**Flags affected**: `n-----z-`

`A` ← `M`

---
`n` ← Most significant bit of Accumulator

`z` ← Set if the Accumulator is zero

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`LDA #const`      | Immediate                 | `$A9`   | 2 / 3 | 2 |
`LDA addr`        | Absolute                  | `$AD`   | 3     | 4 |
`LDA long`        | Absolute Long             | `$AF`   | 4     | 5 |
`LDA dp`          | Direct Page               | `$A5`   | 2     | 3 |
`LDA (dp)`        | Direct Page Indirect      | `$B2`   | 2     | 5 |
`LDA [dp]`        | Direct Page Indirect Long | `$A7`   | 2     | 6 |
`LDA addr, X`     | Absolute Indexed, X       | `$BD`   | 3     | 4 |
`LDA long, X`     | Absolute Long Indexed, X  | `$BF`   | 4     | 5 |
`LDA addr, Y`     | Absolute Indexed, Y       | `$B9`   | 3     | 4 |
`LDA dp, X`       | Direct Page Indexed, X    | `$B5`   | 2     | 4 |
`LDA (dp, X)`     | Direct Page Indirect, X   | `$A1`   | 2     | 6 |
`LDA (dp), Y`     | DP Indirect Indexed, Y    | `$B1`   | 2     | 5 |
`LDA [dp], Y`     | DP Indirect Long Indexed, Y | `$B7` | 2     | 6 |
`LDA sr, S`       | Stack Relative            | `$A3`   | 2     | 4 |
`LDA (sr, S), Y`  | SR Indirect Indexed, Y    | `$B3`   | 2     | 7 |



LDX - Load Index Register X from Memory
=======================================

**Flags affected**: `n-----z-`

`X` ← `M`

---
`n` ← Most significant bit of X

`z` ← Set if the X is zero

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`LDX #const`      | Immediate                 | `$A2`   | 2 / 3 | 2 |
`LDX addr`        | Absolute                  | `$AE`   | 3     | 4 |
`LDX dp`          | Direct Page               | `$A6`   | 2     | 3 |
`LDX addr, Y`     | Absolute Indexed, Y       | `$BE`   | 3     | 4 |
`LDX dp, Y`       | Direct Page Indexed, Y    | `$B6`   | 2     | 4 |



LDY - Load Index Register Y from Memory
=======================================

**Flags affected**: `n-----z-`

`Y` ← `M`

---
`n` ← Most significant bit of Y

`z` ← Set if the Y is zero

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`LDY #const`      | Immediate                 | `$A0`   | 2 / 3 | 2 |
`LDY addr`        | Absolute                  | `$AC`   | 3     | 4 |
`LDY dp`          | Direct Page               | `$A4`   | 2     | 3 |
`LDY addr, X`     | Absolute Indexed, X       | `$BC`   | 3     | 4 |
`LDY dp, X`       | Direct Page Indexed, X    | `$B4`   | 2     | 4 |



LSR - Logical Shift Right
=========================

**Flags affected**: `n-----zc`

`M` ← `M >> 1`

---
`n` ← cleared

`z` ← Set if the result is zero

`c` ← Bit 0 of original memory

NOTE: This is an unsigned operation, the MSB of the result is always 0.

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`LSR`             | Accumulator               | `$4A`   | 1     | 2
`LSR addr`        | Absolute                  | `$4E`   | 3     | 6 |
`LSR dp`          | Direct Page               | `$46`   | 2     | 5 |
`LSR addr, X`     | Absolute Indexed, X       | `$5E`   | 3     | 7 |
`LSR dp, X`       | Direct Page Indexed, X    | `$56`   | 2     | 6 |



MVN - Block Move Next
=====================

This instruction is also known as **Block Move Negative**.

**Flags affected**: `--------`

**Parameters:**

`X`: source address

`Y`: destination address

`C`: length - 1

`DBR` ← `destBank`

`repeat:`

`T` ← `srcBank:X`

`DBR:Y` ← `T`

`X` ← `X + 1`

`Y` ← `Y + 1`

`C` ← `C - 1`

`until C == 0xffff`

NOTES:

 * The number of bytes transferred is `C + 1`
 * After the transfer is complete:
    * `DBR` = destination bank
    * `C` = `0xFFFF`
    * `X` = the byte after the end of the source block
    * `Y` = the byte after the end of the destination block.
 * If bit 4 (x) of the status register is set, `MVN` will only be able
   to access the first page of the source and destination banks.
 * Block move instructions can be interrupted.  The move will resume after the
   <abbr title="Interrupt Service Routine">ISR</abbr> returns, provided the
   `C`, `X`, `Y` registers, Program Counter and `MVN` instruction are unchanged.
 * `MVN` should be used if the blocks do not overlap or if the destination address
   is less than (more negative than) the source address.
 * `MVN` can be used to fill an array or memory block:

        set value of first element

        set X = array_start
        set Y = array_start + element_size
        set C = (array_count - 1) * element_size - 1

        MVN array_bank array_bank

---
Syntax                | Addressing Mode           | Opcode| Bytes | Cycles |
----------------------|---------------------------|-------|-------|--------|
`MVN srcBank, destBank` | Block Move                | `$54`   | 3     |  | 7 per byte moved



MVP - Block Move Previous
=========================

This instruction is also known as **Block Move Positive**.

**Flags affected**: `--------`

**Parameters:**

`X`: address of last source byte

`Y`: address of last destination byte

`C`: length - 1

`DBR` ← `destBank`

`repeat:`

`T` ← `srcBank:X`

`DBR:Y` ← `T`

`X` ← `X - 1`

`Y` ← `Y - 1`

`C` ← `C - 1`

`until C == 0xffff`

NOTES:

 * The number of bytes transferred is `C + 1`.
 * After the transfer is complete:
    * `DBR` = destination bank
    * `C` = `0xFFFF`
    * `X` = the byte before the start of the source block
    * `Y` = the byte before the start of the destination block.
 * If bit 4 (x) of the status register is set, `MVP` will only be able
   to access the first page of the source and destination banks.
 * Block move instructions can be interrupted.  The move will resume after the
   <abbr title="Interrupt Service Routine">ISR</abbr> returns, provided the
   `C`, `X`, `Y` registers, Program Counter and `MVP` instruction are unchanged.
 * `MVP` should be used if the blocks could overlap and the destination address
   is greater than (more positive[^mvp-more-positive] than) the source address.

---
Syntax                | Addressing Mode           | Opcode| Bytes | Cycles |
----------------------|---------------------------|-------|-------|--------|
`MVP srcBank, destBank` | Block Move                | `$44`   | 3     |  | 7 per byte moved


[^mvp-more-positive]: MVP more positive source: W65C816S 8/16–bit Microprocessor Datasheet,
    Table 5-7 Instruction Operation, 9b Block Move Positive,
    by Western Design Center, Inc


NOP - No Operation
==================

**Flags affected**: `--------`

&mdash;

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`NOP`             | Implied                   | `$EA`   | 1     | 2 |



ORA - OR Accumulator with Memory
================================

**Flags affected**: `n-----z-`

`A` ← `A | M`

---
`n` ← Most significant bit of result

`z` ← Set if the result is zero

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`ORA #const`      | Immediate                 | `$09`   | 2 / 3 | 2 |
`ORA addr`        | Absolute                  | `$0D`   | 3     | 4 |
`ORA long`        | Absolute Long             | `$0F`   | 4     | 5 |
`ORA dp`          | Direct Page               | `$05`   | 2     | 3 |
`ORA (dp)`        | Direct Page Indirect      | `$12`   | 2     | 5 |
`ORA [dp]`        | Direct Page Indirect Long | `$07`   | 2     | 6 |
`ORA addr, X`     | Absolute Indexed, X       | `$1D`   | 3     | 4 |
`ORA long, X`     | Absolute Long Indexed, X  | `$1F`   | 4     | 5 |
`ORA addr, Y`     | Absolute Indexed, Y       | `$19`   | 3     | 4 |
`ORA dp, X`       | Direct Page Indexed, X    | `$15`   | 2     | 4 |
`ORA (dp, X)`     | Direct Page Indirect, X   | `$01`   | 2     | 6 |
`ORA (dp), Y`     | DP Indirect Indexed, Y    | `$11`   | 2     | 5 |
`ORA [dp], Y`     | DP Indirect Long Indexed, Y | `$17` | 2     | 6 |
`ORA sr, S`       | Stack Relative            | `$03`   | 2     | 4 |
`ORA (sr, S), Y`  | SR Indirect Indexed, Y    | `$13`   | 2     | 7 |



PEA - Push Effective Absolute Address
=====================================

**Flags affected**: `--------`

`S` ← `S - 2`
`[S+2]` ← `addr.h`
`[S+1]` ← `addr.l`

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`PEA addr`        | Stack (Absolute)          | `$F4`   | 3     | 5



PEI - Push Effective Indirect Address
=====================================

**Flags affected**: `--------`

`S` ← `S - 2`
`[S+2]` ← `[0:D+dp+1]`
`[S+1]` ← `[0:D+dp]`

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`PEI (dp)`        | Stack (DP Indirect)       | `$D4`   | 2     | 6 |




PER - Push Effective PC Relative Indirect Address
=================================================

**Flags affected**: `--------`

`S` ← `S - 2`
`T` ← `PC + Label`
`[S+2]` ← `T.h`
`[S+1]` ← `T.l`

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`PER label`       | Stack (PC Relative Long)  | `$62`   | 3     | 6



Push to Stack
=============

**Flags affected**: `--------`

**8 bit register:**
`S` ← `S - 1`
`[S+1]` ← `R`

**16 bit register:**
`S` ← `S - 2`
`[S+2]` ← `R.h`
`[S+1]` ← `R.l`

---
Syntax          | Name                      | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`PHA`             | Push Accumulator          | `$48`   | 1     | 3 |
`PHB`             | Push Data Bank            | `$8B`   | 1     | 3
`PHD`             | Push Direct Page Register | `$0B`   | 1     | 4
`PHK`             | Push Program Bank Register| `$4B`   | 1     | 3
`PHP`             | Push Processor Status Register| `$08`| 1     | 3
`PHX`             | Push Index Register X     | `$DA`   | 1     | 3 |
`PHY`             | Push Index Register Y     | `$5A`   | 1     | 3 |



Pull from Stack
================

**Flags affected**: `n-----z-`

**Flags affected (`PLP`)**: `nvmxdizc`

**8 bit register:**
`R` ← `[S+1]`
`S` ← `S + 1`

---
`n` ← Most significant bit of register
`z` ← Set if the register is zero

---
**16 bit register:**
`R.l` ← `[S+1]`
`R.h` ← `[S+2]`
`S` ← `S + 2`

---
`n` ← Most significant bit of register
`z` ← Set if the register is zero

---
**PLP (Native Mode):**
`P` ← `[S+1]`
`S` ← `S + 1`

---
**PLP (Emulation Mode):**
`P` ← `[S+1]`
`S` ← `S + 1`
`x` ← `1`
`m` ← `1`

Note about PLP: If bit 4 (x) of the status register is set, then the high
byte of the index registers will be set to 0.

---
Syntax          | Name                      | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`PLA`             | Pull Accumulator          | `$68`   | 1     | 4 |
`PLB`             | Pull Data Bank            | `$AB`   | 1     | 4
`PLD`             | Pull Direct Page Register | `$2B`   | 1     | 5
`PLP`             | Pull Processor Status Register| `$28`| 1     | 4
`PLX`             | Pull Index Register X     | `$FA`   | 1     | 4 |
`PLY`             | Pull Index Register Y     | `$7A`   | 1     | 4 |



REP - Reset Status Bits
=======================

**Flags affected**: `nvmxdizc`

**Native Mode:**

`P` ← `P & (~M)`

**Emulation Mode:**

`P` ← `P & (~M)`

`x` ← `1`

`m` ← `1`

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`REP #const`      | Immediate                 | `$C2`   | 2     | 3



ROL - Rotate Left
=================

**Flags affected**: `n-----zc`

`M` ← `M + M + c`

---
`n` ← Most significant bit of result

`z` ← Set if the result is zero

`c` ← Most significant bit of original Memory

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`ROL`             | Accumulator               | `$2A`   | 1     | 2
`ROL addr`        | Absolute                  | `$2E`   | 3     | 6 |
`ROL dp`          | Direct Page               | `$26`   | 2     | 5 |
`ROL addr, X`     | Absolute Indexed, X       | `$3E`   | 3     | 7 |
`ROL dp, X`       | Direct Page Indexed, X    | `$36`   | 2     | 6 |



ROR - Rotate Right
==================

**Flags affected**: `n-----zc`

`M` ← `(c << (m ? 7 : 15)) | (M >> 1)`

---
`n` ← Most significant bit of result

`z` ← Set if the result is zero

`c` ← Bit 0 of original memory

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`ROR`             | Accumulator               | `$6A`   | 1     | 2
`ROR addr`        | Absolute                  | `$6E`   | 3     | 6 |
`ROR dp`          | Direct Page               | `$66`   | 2     | 5 |
`ROR addr, X`     | Absolute Indexed, X       | `$7E`   | 3     | 7 |
`ROR dp, X`       | Direct Page Indexed, X    | `$76`   | 2     | 6 |



RTI - Return From Interrupt
===========================

**Flags affected**: `nvmxdizc`

**Native Mode:**
`P` ← `[S+1]`
`PC.l` ← `[S+2]`
`PC.h` ← `[S+3]`
`PBR` ← `[S+4]`
`S` ← `S + 4`

**Emulation Mode:**
`P` ← `[S+1]`
`x` ← `1`
`m` ← `1`
`PC.l` ← `[S+2]`
`PC.h` ← `[S+3]`
`S` ← `S + 3`

Note: If bit 4 (x) of the status register is set, then the high byte of
the index registers will be set to 0.

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`RTI`             | Stack (return interrupt)  | `$40`   | 1     | 6 |



RTS, RTL - Return From Subroutine
=================================

**Flags affected**: `--------`

**RTS:**
`PC.l` ← `[S+1]`
`PC.h` ← `[S+2]`
`S` ← `S + 2`
`PC` ← `PC + 1`

**RTL:**
`PC.l` ← `[S+1]`
`PC.h` ← `[S+2]`
`PBR` ← `[S+3]`
`S` ← `S + 3`
`PC` ← `PC + 1`


---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`RTS`             | Stack (return)            | `$60`   | 1     | 6
`RTL`             | Stack (return long)       | `$6B`   | 1     | 6



SBC - Subtract with Borrow from Accumulator
===========================================

**Flags affected**: `nv----zc`

`A` ← `A + (~M) + c`


`n` ← Most significant bit of result

`v` ← Signed overflow of result

`z` ← Set if the Accumulator is zero

`c` ← Carry from ALU (bit 8/16 of result) (set if borrow not required)

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`SBC #const`      | Immediate                 | `$E9`   | 2 / 3 | 2 |
`SBC addr`        | Absolute                  | `$ED`   | 3     | 4 |
`SBC long`        | Absolute Long             | `$EF`   | 4     | 5 |
`SBC dp`          | Direct Page               | `$E5`   | 2     | 3 |
`SBC (dp)`        | Direct Page Indirect      | `$F2`   | 2     | 5 |
`SBC [dp]`        | Direct Page Indirect Long | `$E7`   | 2     | 6 |
`SBC addr, X`     | Absolute Indexed, X       | `$FD`   | 3     | 4 |
`SBC long, X`     | Absolute Long Indexed, X  | `$FF`   | 4     | 5 |
`SBC addr, Y`     | Absolute Indexed, Y       | `$F9`   | 3     | 4 |
`SBC dp, X`       | Direct Page Indexed, X    | `$F5`   | 2     | 4 |
`SBC (dp, X)`     | Direct Page Indirect, X   | `$E1`   | 2     | 6 |
`SBC (dp), Y`     | DP Indirect Indexed, Y    | `$F1`   | 2     | 5 |
`SBC [dp], Y`     | DP Indirect Long Indexed, Y | `$F7` | 2     | 6 |
`SBC sr, S`       | Stack Relative            | `$E3`   | 2     | 4 |
`SBC (sr, S), Y`  | SR Indirect Indexed, Y    | `$F3`   | 2     | 7 |



Set Status Flags
================

**Flags affected (`SEC`)**: `-------c`

**Flags affected (`SED`)**: `----d---`

**Flags affected (`SEI`)**: `-----i--`

**SEC:**

`c` ← `1`

**SED:**

`d` ← `1`

**SEI:**

`i` ← `1`
---
Syntax          | Name                      | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`SEC`             | Set Carry Flag            | `$38`   | 1     | 2
`SEI`             | Set Interrupt Disable Flag | `$78`  | 1     | 2
`SED`             | Set Decimal Flag          | `$F8`   | 1     | 2



SEP - Set Status Bits
=====================

**Flags affected**: `nvmxdizc`

`P` ← `P | M`

NOTE: If bit 4 (x) of the status register is set, then the high byte of
the index registers will be set to 0.

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`SEP #const`      | Immediate                 | `$E2`   | 2     | 3



STA - Store Accumulator to Memory
=================================

**Flags affected**: `--------`

`M` ← `A`

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`STA addr`        | Absolute                  | `$8D`   | 3     | 4 |
`STA long`        | Absolute Long             | `$8F`   | 4     | 5 |
`STA dp`          | Direct Page               | `$85`   | 2     | 3 |
`STA (dp)`        | Direct Page Indirect      | `$92`   | 2     | 5 |
`STA [dp]`        | Direct Page Indirect Long | `$87`   | 2     | 6 |
`STA addr, X`     | Absolute Indexed, X       | `$9D`   | 3     | 5 |
`STA long, X`     | Absolute Long Indexed, X  | `$9F`   | 4     | 5 |
`STA addr, Y`     | Absolute Indexed, Y       | `$99`   | 3     | 5 |
`STA dp, X`       | Direct Page Indexed, X    | `$95`   | 2     | 4 |
`STA (dp, X)`     | Direct Page Indirect, X   | `$81`   | 2     | 6 |
`STA (dp), Y`     | DP Indirect Indexed, Y    | `$91`   | 2     | 6 |
`STA [dp], Y`     | DP Indirect Long Indexed, Y | `$97` | 2     | 6 |
`STA sr, S`       | Stack Relative            | `$83`   | 2     | 4 |
`STA (sr, S), Y`  | SR Indirect Indexed, Y    | `$93`   | 2     | 7 |



STP - Stop the Processor
========================

**Flags affected**: `--------`

`CPU clock enabled` ← `0`

Note, this instruction can cause some builds of snes9x to freeze.

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`STP`             | Implied                   | `$DB`   | 1     | 3



STX - Store Index Register X to Memory
======================================

**Flags affected**: `--------`

`M` ← `X`

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`STX addr`        | Absolute                  | `$8E`   | 3     | 4 |
`STX dp`          | Direct Page               | `$86`   | 2     | 3 |
`STX dp, Y`       | Direct Page Indexed, Y    | `$96`   | 2     | 4 |



STY - Store Index Register Y to Memory
======================================

**Flags affected**: `--------`

`M` ← `Y`

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`STY addr`        | Absolute                  | `$8C`   | 3     | 4 |
`STY dp`          | Direct Page               | `$84`   | 2     | 3 |
`STY dp, X`       | Direct Page Indexed, X    | `$94`   | 2     | 4 |



STZ - Store Zero to Memory
==========================

**Flags affected**: `--------`

`M` ← `0`

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`STZ addr`        | Absolute                  | `$9C`   | 3     | 4 |
`STZ dp`          | Direct Page               | `$64`   | 2     | 3 |
`STZ addr, X`     | Absolute Indexed, X       | `$9E`   | 3     | 5 |
`STZ dp, X`       | Direct Page Indexed, X    | `$74`   | 2     | 4 |



Transfer Registers
==================

**Flags affected**: `n-----z-`

**Flags affected (TCS, TXS)**: `--------`

`Rd` ← `Rs`

---
`n` ← Most significant bit of the transferred value

`z` ← Set if the transferred value is zero

The number of bits transferred depends on the state of the m, x and e flags:

 * Accumulator to Index:
    * 8 bit Index (x=1): 8 bits transferred
    * 16 bit Index (x=0): 16 bits transferred, no matter the state of m
 * Accumulator to/from Direct page register (D): 16 bits transferred
 * Index to Accumulator:
    * 8 bit A (m=1): 8 bits transferred
    * 16 bit A (m=0): 16 bits transferred (when x=1 (8 bit index), the high byte is 0)
 * Stack Pointer to X:
    * 8 bit Index (x=1): 8 bits transferred, high byte of X = 0
    * 16 bit Index (x=0): 16 bits transferred
 * Stack Pointer to Accumulator: 16 bits transferred
 * A/X to Stack Pointer:
    * Native mode: 16 bits transferred
    * Emulation mode: 8 bits transferred, high byte of S = 1

---
Syntax          | Name                      | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`TAX`             | Transfer A to X           | `$AA`   | 1     | 2
`TAY`             | Transfer A to Y           | `$A8`   | 1     | 2
`TCD`             | Transfer 16 bit A to D    | `$5B`   | 1     | 2
`TCS`             | Transfer 16 bit A to S    | `$1B`   | 1     | 2
`TDC`             | Transfer D to 16 bit A    | `$7B`   | 1     | 2
`TSC`             | Transfer S to 16 bit A    | `$3B`   | 1     | 2
`TSX`             | Transfer S to X           | `$BA`   | 1     | 2
`TXA`             | Transfer X to A           | `$8A`   | 1     | 2
`TXS`             | Transfer X to S           | `$9A`   | 1     | 2
`TXY`             | Transfer X to Y           | `$9B`   | 1     | 2
`TYA`             | Transfer Y to A           | `$98`   | 1     | 2
`TYX`             | Transfer Y to X           | `$BB`   | 1     | 2




TRB - Test and Reset Memory Bits Against Accumulator
====================================================

**Flags affected**: `------z-`

`z` ← Set if logical AND of memory and Accumulator is zero

---
`M` ← `M & (~A)`

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`TRB addr`        | Absolute                  | `$1C`   | 3     | 6 |
`TRB dp`          | Direct Page               | `$14`   | 2     | 5 |



TSB - Test and Set Memory Bits Against Accumulator
==================================================

**Flags affected**: `------z-`

`z` ← Set if logical AND of memory and Accumulator is zero

---
`M` ← `M | A`

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`TSB addr`        | Absolute                  | `$0C`   | 3     | 6 |
`TSB dp`          | Direct Page               | `$04`   | 2     | 5 |



WAI - Wait for Interrupt
========================

**Flags affected**: `--------`

`RDY pin` ← `0`

`wait for NMI, IRQ or ABORT signal`

`execute interrupt handler if interrupt is not masked`

`RDY pin` ← `1`

When the `RDY` (Ready) pin is pulled low the processor enters a low
power mode.

This instruction is useful when you want as little delay as possible
between the interrupt signal and the processor executing the interrupt
handler.

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`WAI`             | Implied                   | `$CB`   | 1     | 3 | additional cycles needed by interrupt handler to restart the processor



WDM - Reserved for Future Expansion
===================================

**Flags affected**: `--------`

&mdash;

On the SNES it does nothing. This instruction should not be used in your
program.

The bsnes-plus and Mesen-S debuggers have a setting that changes `WDM`
instructions into software breakpoints.

This instruction has a non-standard syntax.  Some assemblers will use `wdm #1`,
while other assemblers use `wdm 1`.

<table>
<thead>
<tr><th>Syntax</th><th>Addressing Mode</th><th>Opcode</th><th>Bytes</th><th>Cycles</th><th>Extra</th>
</tr>
</thead>
<tbody>
<tr><td>WDM #const</td><td rowspan="2"></td><td rowspan="2">$42</td><td rowspan="2">2</td><td rowspan="2">2</td><td rowspan="2"></td></tr>
<tr><td>WDM param</td></tr>
</tbody>
</table>



XBA - Exchange the B and A Accumulators
=======================================

**Flags affected**: `n-----z-`

`T` ← `Ah`

`Ah` ← `Al`

`Al` ← `T`


`n` ← Value of bit 7 of the new Accumulator (even in 16 bit mode)
`z` ← Set if bits 0-7 of the new Accumulator are 0 (even in 16 bit mode)

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`XBA`             | Implied                   | `$EB`   | 1     | 3



XCE - Exchange Carry and Emulation Bits
=======================================

**Flags affected**: `--mx---c : e`

`c` ← Previous `e` flag

`e` ← Previous `c` flag




**if e is set (Emulation Mode):**

`m` ← `1`

`x` ← `1`

`S.h` ← `0x01`

`X.h` ← `0x00`

`Y.h` ← `0x00`

Note: The high byte of the Stack Pointer is fixed in emulation mode.

Note: Emulation mode will set bit 4 (x) of the status register, which will also set the high byte of the index registers to 0.

---
Syntax          | Addressing Mode           | Opcode| Bytes | Cycles |
----------------|---------------------------|-------|-------|--------|
`XCE`             | Implied                   | `$FB`   | 1     | 2


Sources
=======
 * A 65816 Primer, by Brett Tabke
 * W65C816S 8/16–bit Microprocessor Datasheet by The Western Design Center, Inc
 * Programming the 65816, by David Eyes and Ron Lichty
 * [All_About_Your_64 - 65816 Reference](http://www.unusedino.de/ec64/technical/aay/c64/ebmain.htm),
   by Ninja/The Dreams in 1995-2005
 * [higan](https://github.com/higan-emu/higan/) source code,
   [wdc65816 directory](https://github.com/higan-emu/higan/tree/master/higan/component/processor/wdc65816),
   by Near

