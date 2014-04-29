.syntax unified

.section .text.weak_isr.isr_nmi
.weak isr_nmi
isr_nmi:
  bkpt

.section .text.weak_isr.isr_hardfault
.weak isr_hardfault
isr_hardfault:
  bkpt

.section .text.weak_isr.isr_mmfault
.weak isr_mmfault
isr_mmfault:
  bkpt

.section .text.weak_isr.isr_busfault
.weak isr_busfault
isr_busfault:
  bkpt

.section .text.weak_isr.isr_usagefault
.weak isr_usagefault
isr_usagefault:
  bkpt

.section .text.weak_isr.isr_svcall
.weak isr_svcall
isr_svcall:
  bkpt

.section .text.weak_isr.isr_pendsv
.weak isr_pendsv
isr_pendsv:
  bkpt

.section .text.weak_isr.isr_systick
.weak isr_systick
isr_systick:
  bkpt

.section .text.weak_isr.isr_hang
.weak isr_hang
isr_hang:
  b isr_hang
