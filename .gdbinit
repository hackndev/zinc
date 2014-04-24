define tr
target remote localhost:3333
end

define trx
target extended localhost:3333
end

define ld
load build/zinc.elf
end

define rst
monitor jtag_reset
end
