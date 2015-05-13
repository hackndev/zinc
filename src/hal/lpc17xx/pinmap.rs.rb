! // Zinc, the bare metal stack for rust.
! // Copyright 2014 Vladimir "farcaller" Pouzanov <farcaller@gmail.com>
! //
! // Licensed under the Apache License, Version 2.0 (the "License");
! // you may not use this file except in compliance with the License.
! // You may obtain a copy of the License at
! //
! //     http://www.apache.org/licenses/LICENSE-2.0
! //
! // Unless required by applicable law or agreed to in writing, software
! // distributed under the License is distributed on an "AS IS" BASIS,
! // WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
! // See the License for the specific language governing permissions and
! // limitations under the License.
!
! //! Automatically generated file, do not edit
! //! Update the definition in pinmap.rs.rb and re-generate pinmap.rs with
! //! support/pingen.rb <src> <dst>
! //!
! //! This module provides all possible pin configurations for LPC17xx.
!

port 0 {
  RD1[can rx 1]          TXD3[uart tx 3]       SDA1[i2c data 1];
  TD1[can tx 1]          RXD3[uart rx 3]       SCL1[i2c clock 1];
  TXD0[uart tx 0]        AD0_7[adc]            *;
  RXD0[uart rx 0]        AD0_6[adc]            *;
  I2SRX_CLK[i2s rx clk]  RD2[can rx 2]         CAP2_0[timer cap 2];
  I2SRX_WS[i2s rx ws]    TD2[can tx 2]         CAP2_1[timer cap 2];
  I2SRX_SDA[i2s rx sda]  SSEL1[ssp cs 1]       MAT2_0[timer mat 2];
  I2STX_CLK[i2s tx clk]  SCK1[ssp sck 1]       MAT2_1[timer mat 2];
  I2STX_WS[i2s tx ws]    MISO1[ssp miso 1]     MAT2_2[timer mat 2];
  I2STX_SDA[i2s tx sda]  MOSI1[ssp mosi 1]     MAT2_3[timer mat 2];
  TXD2[uart tx 2]        SDA2[i2c data 2]      MAT3_0[timer mat 2];
  RXD2[uart rx 2]        SCL2[i2c clock 2]     MAT3_1[timer mat 2];
  *;
  *;
  *;
  TXD1[uart tx 1]        SCK0[ssp sck 0]       SCK[spi sck];

  RXD1[uart rx 1]        SSEL0[ssp cs 0]       SSEL[spi cs];
  CTS1[uart cts 1]       MISO0[ssp miso 0]     MISO[spi miso];
  DCD1[uart dcd 1]       MOSI0[ssp mosi 0]     MOSI[spi mosi];
  DSR1[uart dsr 1]       *                     SDA1[i2c data 1];
  DTR1[uart dtr 1]       *                     SCL1[i2c clock 1];
  RI1[uart ri 1]         *                     RD1[can rx 1];
  RTS1[uart rts 1]       *                     TD1[can tx 1];
  AD0_0[adc]             I2SRX_CLK[i2s rx clk] CAP3_0[timer cap 3];
  AD0_1[adc]             I2SRX_WS[i2s rx ws]   CAP3_1[timer cap 3];
  AD0_2[adc]             I2SRX_SDA[i2s rx sda] TXD3[uart tx 3];
  AD0_3[adc]             AOUT[dac]             TXD3[uart rx 3];
  SDA0[i2c data 0]       USB_SDA[usb i2c data] *;
  SCL0[i2c clock 0]      USB_SCL[usb i2c clock] *;
  USB_D_pos[usb d pos]   *                     *;
  USB_D_neg[usb d neg]   *                     *;
  *;
}

port 1 {
  ENET_TXD0[eth tx 0]       *                  *;
  ENET_TXD1[eth tx 1]       *                  *;
  *;
  *;
  ENET_TX_EN[eth tx en]     *                  *;
  *;
  *;
  *;
  ENET_CRS[eth crs]         *                  *;
  ENET_RXD0[eth rx 0]       *                  *;
  ENET_RXD1[eth rx 1]       *                  *;
  *;
  *;
  *;
  ENET_RX_ER[eth rx er]     *                     *;
  ENET_REF_CLCK[eth refclk] *                     *;

  ENET_MDC[eth mdc]         *                     *;
  ENET_MDIO[eth mdio]       *                     *;
  USB_UP_LED[usb upled]     PWM1_1[pwm 1]         CAP1_0[timer cap 1];
  MCOA0[motor 0 out a]      USB_PPWR[usb ppwr]    CAP1_1[timer cap 1];
  MCI0[motor 0 in]          PWM1_2[pwm 1]         SCK0[ssp sck 0];
  MCABORT[motor abort]      PWM1_3[pwm 1]         SSEL0[ssp cs 0];
  MCOB0[motor 0 out b]      USB_PWRD[usb pwrd]    MAT1_0[timer mat 1];
  MCI1[motor 1 in]          PWM1_4[pwm 1]         MISO0[ssp miso 0];
  MCI2[motor 2 in]          PWM1_5[pwm 1]         MOSI0[ssp mosi 0];
  MCOA1[motor 1 out a]      *                     MAT1_1[timer mat 1];
  MCOB1[motor 1 out b]      PWM1_6[pwm 1]         CAP0_0[timer cap 0];
  CLKOUT[clkout]            USB_OVRCR[usb overcr] CAP0_1[timer cap 0];
  MCOA2[motor 2 out a]      PCAP1_0[cap]          MAT0_0[timer mat 0];
  MCOB2[motor 2 out b]      PCAP1_1[cap]          MAT0_1[timer mat 0];
  *                         Vbus[vbus]            AD0_4[adc];
  *                         SCK1[ssp sck 1]       AD0_5[adc];
}

port 2 {
  PWM1_1[pwm 1]            TXD1[uart tx 1]    *;
  PWM1_2[pwm 1]            RXD1[uart rx 1]    *;
  PWM1_3[pwm 1]            CTS1[uart cts 1]   *;
  PWM1_4[pwm 1]            DCD1[uart dcd 1]   *;
  PWM1_5[pwm 1]            DSR1[uart dsr 1]   *;
  PWM1_6[pwm 1]            DTR1[uart dtr 1]   *;
  PCAP1_0[cap]             RI1[uart ri 1]     *;
  RD2[can rx 2]            RTS1[uart rts 1]   *;
  TD2[can tx 2]            TXD2[uart tx 2]    ENET_MDC[eth mdc];
  USB_CONNECT[usb connect] RXD2[uart rx 2]    ENET_MDIO[eth mdio];
  EINT0[eint 0]            NMI[nmi]           *;
  EINT1[eint 1]            *                  I2STX_CLK[i2s tx clk];
  EINT2[eint 2]            *                  I2STX_WS[i2s tx ws];
  EINT3[eint 3]            *                  I2STX_SDA[i2s tx sda];
  *;
  *;
}

port 3 {
  *; *; *; *; *; *; *; *; *; *; *; *; *; *; *; *;

  *; *; *; *; *; *; *; *; *;
  *                      MAT0_0[timer mat 0]  PWM1_2[pwm 1];
  STCLK[systick]         MAT0_1[timer mat 0]  PWM1_3[pwm 1];
}

port 4 {
  *; *; *; *; *; *; *; *; *; *; *; *; *; *; *; *;

  *; *; *; *; *; *; *; *; *; *; *; *;
  RX_MCLK[i2s rx mclk]   MAT2_0[timer mat 2]  TXD3[uart tx 3];
  TX_MCLK[i2s tx mclk]   MAT2_1[timer mat 2]  RXD3[uart rx 3];
}
