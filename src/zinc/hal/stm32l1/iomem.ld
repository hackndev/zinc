INCLUDE ./src/zinc/hal/cortex_m3/armmem.ld

/* Derived from:
http://www.st.com/web/en/resource/technical/document/application_note/DM00032987.pdf
*/

stm32l1_iomem_TIM2  = 0x40000000;

stm32l1_iomem_PWR   = 0x40007000;

stm32l1_iomem_FLASH = 0x40023C00;
stm32l1_iomem_RCC   = 0x40023800;

stm32l1_iomem_GPIOA = 0x40020000;
stm32l1_iomem_GPIOB = 0x40020400;
stm32l1_iomem_GPIOC = 0x40020800;
stm32l1_iomem_GPIOD = 0x40020c00;
stm32l1_iomem_GPIOE = 0x40021000;
stm32l1_iomem_GPIOF = 0x40021800;
stm32l1_iomem_GPIOG = 0x40021C00;
stm32l1_iomem_GPIOH = 0x40021400;
