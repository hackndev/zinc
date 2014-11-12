PROVIDE(isr_wdt           = isr_hardfault);
PROVIDE(isr_timer_0       = isr_hardfault);
PROVIDE(isr_timer_1       = isr_hardfault);
PROVIDE(isr_timer_2       = isr_hardfault);
PROVIDE(isr_timer_3       = isr_hardfault);
PROVIDE(isr_uart_0        = isr_hardfault);
PROVIDE(isr_uart_1        = isr_hardfault);
PROVIDE(isr_uart_2        = isr_hardfault);
PROVIDE(isr_uart_3        = isr_hardfault);
PROVIDE(isr_pwm_1         = isr_hardfault);
PROVIDE(isr_i2c_0         = isr_hardfault);
PROVIDE(isr_i2c_1         = isr_hardfault);
PROVIDE(isr_i2c_2         = isr_hardfault);
PROVIDE(isr_spi           = isr_hardfault);
PROVIDE(isr_ssp_0         = isr_hardfault);
PROVIDE(isr_ssp_1         = isr_hardfault);
PROVIDE(isr_pll_0         = isr_hardfault);
PROVIDE(isr_rtc           = isr_hardfault);
PROVIDE(isr_eint_0        = isr_hardfault);
PROVIDE(isr_eint_1        = isr_hardfault);
PROVIDE(isr_eint_2        = isr_hardfault);
PROVIDE(isr_eint_3        = isr_hardfault);
PROVIDE(isr_adc           = isr_hardfault);
PROVIDE(isr_bod           = isr_hardfault);
PROVIDE(isr_usb           = isr_hardfault);
PROVIDE(isr_can           = isr_hardfault);
PROVIDE(isr_dma           = isr_hardfault);
PROVIDE(isr_i2s           = isr_hardfault);
PROVIDE(isr_enet          = isr_hardfault);
PROVIDE(isr_rit           = isr_hardfault);
PROVIDE(isr_mcpwm         = isr_hardfault);
PROVIDE(isr_qei           = isr_hardfault);
PROVIDE(isr_pll_1         = isr_hardfault);
PROVIDE(isr_usb_activity  = isr_hardfault);
PROVIDE(isr_can_activity  = isr_hardfault);

lpc17xx_iomem_GPIO0     = 0x2009C000;
lpc17xx_iomem_GPIO1     = 0x2009C020;
lpc17xx_iomem_GPIO2     = 0x2009C040;
lpc17xx_iomem_GPIO3     = 0x2009C060;
lpc17xx_iomem_GPIO4     = 0x2009C080;

lpc17xx_iomem_TIMER0    = 0x40004000;
lpc17xx_iomem_TIMER1    = 0x40008000;

lpc17xx_iomem_UART0     = 0x4000C000;

lpc17xx_iomem_PINSEL0   = 0x4002C000;
lpc17xx_iomem_PINSEL1   = 0x4002C004;
lpc17xx_iomem_PINSEL2   = 0x4002C008;
lpc17xx_iomem_PINSEL3   = 0x4002C00C;
lpc17xx_iomem_PINSEL4   = 0x4002C010;
lpc17xx_iomem_PINSEL7   = 0x4002C01C;
lpc17xx_iomem_PINSEL9   = 0x4002C024;
lpc17xx_iomem_PINSEL10  = 0x4002C028;

lpc17xx_iomem_SSP1      = 0x40030000;
lpc17xx_iomem_SSP0      = 0x40088000;

lpc17xx_iomem_TIMER2    = 0x40090000;
lpc17xx_iomem_TIMER3    = 0x40094000;

lpc17xx_iomem_UART2     = 0x40098000;
lpc17xx_iomem_UART3     = 0x4009C000;

lpc17xx_iomem_FLASHCFG  = 0x400FC000;

lpc17xx_iomem_PLL0CON   = 0x400FC080;
lpc17xx_iomem_PLL0CFG   = 0x400FC084;
lpc17xx_iomem_PLL0STAT  = 0x400FC088;
lpc17xx_iomem_PLL0FEED  = 0x400FC08C;

lpc17xx_iomem_PCONP     = 0x400FC0C4;

lpc17xx_iomem_CCLKCFG   = 0x400FC104;
lpc17xx_iomem_CLKSRCSEL = 0x400FC10C;

lpc17xx_iomem_PCLKSEL0  = 0x400FC1A8;
lpc17xx_iomem_PCLKSEL1  = 0x400FC1AC;

lpc17xx_iomem_SCS       = 0x400FC1A0;
