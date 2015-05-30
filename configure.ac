AC_INIT(zinc, 0.1)

cross_compiling=yes

AC_CHECK_TOOL(GCC, gcc, :)
AC_CHECK_TOOL(STRIP, strip, :)
AC_CHECK_TOOL(OBJCOPY, objcopy, :)
AC_CHECK_TOOL(OBJDUMP, objdump, :)
AC_CHECK_TOOL(AR, ar, :)

AC_SUBST(CC, $GCC)

AC_ARG_VAR(PLATFORM, [Platform for which to build])

case $PLATFORM in
lpc17xx | stm32l1)
	platform_target=thumbv7m-none-eabi
	;;
stm32f4 | k20 | tiva_c)
	platform_target=thumbv7em-none-eabi
	;;
*)
	AC_MSG_ERROR([Unknown platform $PLATFORM])
	;;
esac

AC_SUBST(TARGET, $platform_target)

AC_CONFIG_FILES([Makefile])
AC_CONFIG_FILES([.cargo/config:cargo_config.in])

AC_OUTPUT
