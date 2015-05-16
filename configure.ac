AC_INIT(zinc, 0.1)

cross_compiling=yes

AC_CHECK_TOOL(GCC, gcc, :)
AC_CHECK_TOOL(STRIP, strip, :)
AC_CHECK_TOOL(OBJCOPY, objcopy, :)
AC_CHECK_TOOL(OBJDUMP, objdump, :)
AC_CHECK_TOOL(AR, ar, :)

AC_SUBST(host, $host)
AC_SUBST(CC, $GCC)

AC_ARG_VAR(MCU, [MCU type])

case $MCU in
lpc17xx)
	tgt_cpu=cortex-m3
	;;
stm32f4)
	tgt_cpu=cortex-m4
	;;
stm32l1)
	tgt_cpu=cortex-m3
	;;
k20)
	tgt_cpu=cortex-m4
	;;
tiva_c)
	tgt_cpu=cortex-m4
	;;
*)
	AC_MSG_ERROR([Unknown MCU $MCU])
	;;
esac

tgt_arch=arm
tgt_layout=e-m:e-p:32:32-i1:8:32-i8:8:32-i16:16:32-i64:64-v128:64:128-a:0:32-n32-S64
tgt_os=none
tgt_endian=little
tgt_ptr_width=32
tgt_pre_link_args=\"-mthumb\",\ \"-lm\",\ \"-lgcc\",\ \"-mcpu=$tgt_cpu\"

case $tgt_cpu in
cortex-m3)
	tgt_llvm_tgt=thumbv7em-none-eabi
	;;
cortex-m4)
	tgt_llvm_tgt=thumbv7m-none-eabi
	;;
esac

AC_SUBST(tgt_arch, $tgt_arch)
AC_SUBST(tgt_cpu, $tgt_cpu)
AC_SUBST(tgt_layout, $tgt_layout)
AC_SUBST(tgt_llvm_tgt, $tgt_llvm_tgt)
AC_SUBST(tgt_os, $tgt_os)
AC_SUBST(tgt_endian, $tgt_endian)
AC_SUBST(tgt_ptr_width, $tgt_ptr_width)
AC_SUBST(tgt_pre_link_args, $tgt_pre_link_args)

rustc_version=$(rustc --version|awk '{sub(/\(/, "", $3); print $3}')

AC_CONFIG_COMMANDS([libcore.rlib],
		   [mkdir -p rust]
		   [pushd rust]
		   [wget -O rust.tar.gz https://github.com/rust-lang/rust/tarball/$commit]
		   [tar -zx --strip-components=1 -f rust.tar.gz]
		   [ln -s ../$host.json $host.json]
		   [rustc -C opt-level=2 -C ar=$tgt_ar -Z no-landing-pads --target $host -g src/libcore/lib.rs --out-dir ..]
		   [popd],
		   [tgt_llvm_tgt=$tgt_llvm_tgt]
		   [commit=$rustc_version]
		   [tgt_ar=$AR]
		   [host=$host])

AC_CONFIG_FILES([Makefile])

if test -n "$host"; then
	AC_CONFIG_FILES([$host.json:target.json.in])
fi

AC_OUTPUT
