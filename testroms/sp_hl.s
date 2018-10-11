; Simplest possible test to test SP,HL operations

; Add in the header.
.name "sub test"
.licenseecodenew "ZZ"
.cartridgetype 0
.countrycode $01
.nintendologo
.version $00
.computegbchecksum
.computegbcomplementcheck

; Set up the ROM parameters.
.ramsize 0
.rombanksize $4000
.rombanks 2
.romdmg


.MEMORYMAP
SLOTSIZE $4000
DEFAULTSLOT 0
SLOT 0 $0000
SLOT 1 $4000
.ENDME

.ROMBANKMAP
BANKSTOTAL 2
BANKSIZE $4000
BANKS 2
.ENDRO

.BANK 0 SLOT 1
.ORGA $4000


.bank 0 .slot 0
.org $100
.section "Header" force
  nop
  jp $150
.ends

.bank 0 .slot 0
.org $150

ld sp,$0000
add sp,$00
nop
ld bc,$0000
push bc
pop af
nop

ld sp,$00FF
add sp,$01
nop
ld bc,$0000
push bc
pop af
nop

ld sp,$00F0
add sp,$10
nop
ld bc,$0000
push bc
pop af
nop

ld sp,$FFF0
add sp,$10
nop
ld bc,$0000
push bc
pop af
nop


ld sp,$0FFF
add sp,$01
nop
ld bc,$0000
push bc
pop af
nop


ld sp,$0001
add sp,$0F ; wla-dx doesn't want to let me use $FF here. =/
nop
ld bc,$0000
push bc
pop af
nop

