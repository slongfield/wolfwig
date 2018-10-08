; Simplest posible test rom to test subtraction.

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

ld a,$00
sub $00
nop
ld a,$FF
sub $00
nop
ld a,$00
sub $FF
nop
ld a,$FF
sub $FF
nop
nop
ld a,$0F
sub $00
nop
ld a,$00
sub $0F
nop
ld a,$0F
sub $0F
nop
nop
ld a,$F0
sub $00
nop
ld a,$00
sub $F0
nop
ld a,$F0
sub $F0

