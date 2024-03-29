dnl This file is part of SoundTouch, an audio processing library for pitch/time adjustments
dnl
dnl SoundTouch is free software; you can redistribute it and/or modify it under the
dnl terms of the GNU General Public License as published by the Free Software
dnl Foundation; either version 2 of the License, or (at your option) any later
dnl version.
dnl
dnl SoundTouch is distributed in the hope that it will be useful, but WITHOUT ANY
dnl WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
dnl FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more
dnl details.
dnl
dnl You should have received a copy of the GNU General Public License along with
dnl this program; if not, write to the Free Software Foundation, Inc., 59 Temple
dnl Place - Suite 330, Boston, MA  02111-1307, USA
# Process this file with autoconf to produce a configure script.

AC_INIT([SoundTouch],[2.3.2],[http://www.surina.net/soundtouch])
dnl Default to libSoundTouch.so.$LIB_SONAME.0.0
LIB_SONAME=1
AC_SUBST(LIB_SONAME)

AC_CONFIG_AUX_DIR(config)
AC_CONFIG_MACRO_DIR([config/m4])
AC_CONFIG_HEADERS([config.h include/soundtouch_config.h])
AM_INIT_AUTOMAKE
AM_SILENT_RULES([yes])
#AC_DISABLE_SHARED	dnl This makes libtool only build static libs
AC_DISABLE_STATIC	dnl This makes libtool only build shared libs
#AC_USE_SYSTEM_EXTENSIONS 		dnl enable posix extensions in glibc

AC_LANG(C++)

# Compiler flags. Apply -Ofast (implies -O3 -ffast-math) to allow gcc autovectorization
# generate effective SIMD code.
CXXFLAGS="${CXXFLAGS} -Ofast -Wall -Wextra -Wzero-as-null-pointer-constant -Wno-unknown-pragmas"

# Set AR_FLAGS to avoid build warning "ar: `u' modifier ignored since `D' is the default (see `U')"
AR_FLAGS='cr'


dnl ############################################################################
dnl # Checks for programs                                                      #
dnl ############################################################################
AC_PROG_CXX
#AC_PROG_AWK
AC_PROG_CC
AC_PROG_CPP
AC_PROG_CXXCPP
AC_PROG_INSTALL
#AC_PROG_LN_S
AC_PROG_MAKE_SET

LT_INIT dnl turn on using libtool




dnl ############################################################################
dnl # Checks for header files                                                  #
dnl ############################################################################

#AC_HEADER_SYS_WAIT
#	add any others you want to check for here
AC_CHECK_HEADERS([cpuid.h])
AC_CHECK_HEADERS([arm_neon.h])

if test "x$ac_cv_header_cpuid_h" = "xno"; then
	AC_MSG_WARN([The cpuid.h file was not found therefore the x86 optimizations will be disabled.])
	AC_MSG_WARN([If using a x86 architecture and optimizations are desired then please install gcc (>= 4.3).])
	AC_MSG_WARN([If using a non-x86 architecture then this is expected and can be ignored.])
fi


dnl ############################################################################
dnl # Checks for typedefs, structures, and compiler characteristics            $
dnl ############################################################################
AC_C_CONST
AC_C_INLINE
#AC_TYPE_OFF_T
#AC_TYPE_SIZE_T


AC_ARG_ENABLE(integer-samples,
              [AS_HELP_STRING([--enable-integer-samples],[use integer samples instead of floats [default=no]])],,
              [enable_integer_samples=no])


AC_ARG_ENABLE(openmp,
              [AS_HELP_STRING([--enable-openmp],[use parallel multicore calculation through OpenMP [default=no]])],,
              [enable_openmp=no])

# Let the user enable/disable the x86 optimizations.
# Useful when compiling on non-x86 architectures.
AC_ARG_ENABLE([x86-optimizations],
              [AS_HELP_STRING([--enable-x86-optimizations],
                              [use MMX or SSE optimization [default=yes]])],[enable_x86_optimizations="${enableval}"],
              [enable_x86_optimizations=yes])

# Let the user enable/disable the x86 optimizations.
# Useful when compiling on non-x86 architectures.
AC_ARG_ENABLE([neon-optimizations],
              [AS_HELP_STRING([--enable-neon-optimizations],
                              [use ARM NEON optimization [default=yes]])],[enable_neon_optimizations="${enableval}"],
              [enable_neon_optimizations=yes])


# Tell the Makefile.am if the user wants to disable optimizations.
# Makefile.am will enable them by default if support is available.
# Note: We check if optimizations are supported a few lines down.
AM_CONDITIONAL([X86_OPTIMIZATIONS], [test "x$enable_x86_optimizations" = "xyes"])


if test "x$enable_integer_samples" = "xyes"; then
        echo "****** Integer sample type enabled ******"
        AC_DEFINE(SOUNDTOUCH_INTEGER_SAMPLES,1,[Use Integer as Sample type])
else
        echo "****** Float sample type enabled ******"
        AC_DEFINE(SOUNDTOUCH_FLOAT_SAMPLES,1,[Use Float as Sample type])
fi


if test "x$enable_openmp" = "xyes"; then
        echo "****** openmp optimizations enabled ******"
        AM_CXXFLAGS="-fopenmp $AM_CXXFLAGS"
else
        echo "****** openmp optimizations disabled ******"
fi


# Check if optimizations are supported in the system at build time.
if test "x$enable_x86_optimizations" = "xyes" -a "x$ac_cv_header_cpuid_h" = "xyes"; then
        echo "****** x86 optimizations enabled ******"

	original_saved_CXXFLAGS=$CXXFLAGS
	have_mmx_intrinsics=no
	CXXFLAGS="-mmmx -Winline $CXXFLAGS"

	# Check if the user can compile MMX code using intrinsics.
	# GCC supports MMX intrinsics since version 3.3
	# A more recent GCC (>= 4.3) is recommended.
	AC_COMPILE_IFELSE([AC_LANG_SOURCE([[
	#if defined(__GNUC__) && (__GNUC__ < 3 || (__GNUC__ == 3 && __GNUC_MINOR__ < 3))
	#error "Need GCC >= 3.3 for MMX intrinsics"
	#endif
	#include <mmintrin.h>
	int main () {
	    __m64 loop = _mm_cvtsi32_si64 (1);
	    return _mm_cvtsi64_si32 (loop);
	}]])],[have_mmx_intrinsics=yes])
	CXXFLAGS=$original_saved_CXXFLAGS

	# Inform the user if we did or did not find MMX support.
	#
	# If we enable optimization and integer samples we only require MMX.
	# Disable optimizations in the SSTypes.h file if this is not the case.
	if test "x$have_mmx_intrinsics" = "xyes" ; then
		echo "****** MMX support found ******"
	else
		echo "****** No MMX support found ******"
		if test "x$enable_integer_samples" = "xyes"; then
		        echo "****** Disabling optimizations. Using integer samples with no MMX support ******"
		        CPPFLAGS="-DSOUNDTOUCH_DISABLE_X86_OPTIMIZATIONS $CPPFLAGS"
		fi
	fi


	# SSE support
	original_saved_CXXFLAGS=$CXXFLAGS
	have_sse_intrinsics=no
	CXXFLAGS="-msse -Winline $CXXFLAGS"

	# Check if the user can compile SSE code using intrinsics.
	# GCC supports SSE intrinsics since version 3.3
	# A more recent GCC (>= 4.3) is recommended.
	AC_COMPILE_IFELSE([AC_LANG_SOURCE([[
	#if defined(__GNUC__) && (__GNUC__ < 3 || (__GNUC__ == 3 && __GNUC_MINOR__ < 3))
	#error "Need GCC >= 3.3 for SSE intrinsics"
	#endif
	#include <xmmintrin.h>
	int main () {
	    _mm_setzero_ps();
	    return 0;
	}]])],[have_sse_intrinsics=yes])
	CXXFLAGS=$original_saved_CXXFLAGS

	# Inform the user if we did or did not find SSE support.
	#
	# If we enable optimization and float samples we only require SSE.
	# Disable optimizations in the SSTypes.h file if this is not the case.
	if test "x$have_sse_intrinsics" = "xyes" ; then
		echo "****** SSE support found ******"
	else
		echo "****** No SSE support found ******"
		if test "x$enable_integer_samples" != "xyes"; then
		        echo "****** Disabling optimizations. Using float samples with no SSE support ******"
		        CPPFLAGS="-DSOUNDTOUCH_DISABLE_X86_OPTIMIZATIONS $CPPFLAGS"
		fi
	fi

else
	# Disable optimizations in SSTypes.h since the user requested it.
        echo "****** x86 optimizations disabled ******"
        CPPFLAGS="-DSOUNDTOUCH_DISABLE_X86_OPTIMIZATIONS $CPPFLAGS"
fi


if test "x$enable_neon_optimizations" = "xyes" -a "x$ac_cv_header_arm_neon_h" = "xyes"; then

	# Check for ARM NEON support
	original_saved_CXXFLAGS=$CXXFLAGS
	have_neon=no
	CXXFLAGS="-mfpu=neon -march=native $CXXFLAGS"

	# Check if can compile neon code using intrinsics, require GCC >= 4.3 for autovectorization.
	AC_COMPILE_IFELSE([AC_LANG_SOURCE([[
	#if defined(__GNUC__) && (__GNUC__ < 4 || (__GNUC__ == 4 && __GNUC_MINOR__ < 3))
	#error "Need GCC >= 4.3 for neon autovectorization"
	#endif
	#include <arm_neon.h>
	int main () {
		int32x4_t t = {1};
		return vaddq_s32(t,t)[0] == 2;
	}]])],[have_neon=yes])
	CXXFLAGS=$original_saved_CXXFLAGS
	if test "x$have_neon" = "xyes" ; then
		echo "****** NEON support enabled ******"
		CPPFLAGS="-mfpu=neon -march=native -mtune=native $CPPFLAGS"
		AC_DEFINE(SOUNDTOUCH_USE_NEON,1,[Use ARM NEON extension])
	fi
fi


AC_CANONICAL_HOST
HOST_OS=""
AS_CASE([$host_cpu],
        [x86_64],
        [
            x86_64=true
            x86=true
        ],
        [i?86],
        [
            x86=true
        ])

AM_CONDITIONAL([X86], [test "$x86" = true])
AM_CONDITIONAL([X86_64], [test "$x86_64" = true])

AC_SUBST([HOST_OS])


# Set AM_CXXFLAGS
AC_SUBST([AM_CXXFLAGS], [$AM_CXXFLAGS])

# Empty default CXXFLAGS so user can set them if desirable
#AC_SUBST([CXXFLAGS], [ ])


# SSTypes.h by default enables optimizations. Those already got disabled if
# the user requested for it or if the system does not support them.
#
# Now tell the Makefile.am the optimizations that are supported.
# Note:
# Makefile.am already knows if the user asked for optimizations. We apply
# optimizations by default (if support is available) and then disable all of
# them if the user requested it.
AM_CONDITIONAL([HAVE_MMX], [test "x$have_mmx_intrinsics" = "xyes"])
AM_CONDITIONAL([HAVE_SSE], [test "x$have_sse_intrinsics" = "xyes"])


dnl ############################################################################
dnl # Checks for library functions/classes                                     #
dnl ############################################################################

dnl make -lm get added to the LIBS
AC_CHECK_LIB(m, sqrt,,AC_MSG_ERROR([compatible libc math library not found]))

dnl add whatever functions you might want to check for here
#AC_CHECK_FUNCS([floor ftruncate memmove memset mkdir modf pow realpath sqrt strchr strdup strerror strrchr strstr strtol])







dnl ############################################################################
dnl # Internationaliation and Localiation                                    #
dnl ############################################################################
#AM_GNU_GETTEXT_VERSION([0.11.5])
#AM_GNU_GETTEXT([external])





dnl ############################################################################
dnl # Final                                                                    #
dnl ############################################################################

AC_CONFIG_FILES([
	Makefile
	source/Makefile
		source/SoundTouch/Makefile
		source/SoundStretch/Makefile
        source/SoundTouchDLL/Makefile
	include/Makefile
])

AC_CONFIG_FILES([soundtouch.pc
])
AC_OUTPUT

dnl use 'echo' to put stuff here if you want a message to the builder
