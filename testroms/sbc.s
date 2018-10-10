; Simplest posible test rom to test subtraction with carry

; Add in the header.
.name "sbc test"
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

; 0 - 0, no carry
ld bc,$0000
push bc
pop af
sbc $00
nop
; 0 - 0, carry
ld bc,$0010
push bc
pop af
sbc $00
nop
; 0F - 0, no carry
ld bc,$0F00
push bc
pop af
sbc $00
nop
; 0F - 0, carry
ld bc,$0F10
push bc
pop af
sbc $00
nop
; 0F - 0F, no carry
ld bc,$0F00
push bc
pop af
sbc $0F
nop
; 0F - 0F, carry
ld bc,$0F10
push bc
pop af
sbc $0F
nop
; 7F - 00, carry
ld bc,$0010
push bc
pop af
sbc $7F
nop
; 7F - 80, nc
ld bc,$7F00
push bc
pop af
sbc $80
nop
; 7F - 80, carry
ld bc,$7F10
push bc
pop af
sbc $80
nop
; 7F - FF, nc
ld bc,$7F00
push bc
pop af
sbc $FF
nop
; 5 - 2, carry
ld bc,$0510
push bc
pop af
sbc $2
nop

