#![feature(phase)]
#[phase(plugin)] extern crate macro_ioreg;
extern crate core;

ioregs!(
    group FTM {
        0x0  => SC: u32 "Status and control register"
        {
             0..2 => PS:     uint       "Prescale"
             3..4 => CLKS:   enum { NO_CLOCK=0x0, SYSTEM_CLOCK=0x1, FIXED_FREQ=0x2, EXTERNAL=0x3, }
             5    => CPWMS:  bool
             6    => TOIE:   bool
             7    => TOF:    ro bool
        }
    
        0x4  => CNT: u32 "Count register"
        {
            0..15 => COUNT:  uint
        }
    
        0x8  => MOD: u32 "Modulo register"
        {
            0..15 => MOD:    uint
        }
    
        group Channel {
            0x0 => CSC:    u32         "Compare/capture channel status and control register"
            {
                0 => DMA:    bool
                2 => ELSA:   bool
                3 => ELSB:   bool
                4 => MSA:    bool
                5 => MSB:    bool
                6 => CHIE:   bool
                7 => CHF:    bool
            }
    
            0x4 => CV:     u32
            {
                0..15 => VAL:    uint
            }
        }
    
        0xc  =>      CHANNELS: Channel[8]   "Compare/capture channels"
    
        0x4c =>      CNTIN: u32             "Counter initial value register"
        {
            0..15 => INIT:     uint
        }
    
        0x50 =>      STATUS: u32            "Channel status register"
        {
            0..7 => CHF:      bool[8]
        }
    }
)
