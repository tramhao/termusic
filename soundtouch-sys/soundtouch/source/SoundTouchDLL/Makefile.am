## Process this file with automake to create Makefile.in
##
## This file is part of SoundTouch, an audio processing library for pitch/time adjustments
##
## SoundTouch is free software; you can redistribute it and/or modify it under the
## terms of the GNU General Public License as published by the Free Software
## Foundation; either version 2 of the License, or (at your option) any later
## version.
##
## SoundTouch is distributed in the hope that it will be useful, but WITHOUT ANY
## WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
## A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
##
## You should have received a copy of the GNU General Public License along with
## this program; if not, write to the Free Software Foundation, Inc., 59 Temple
## Place - Suite 330, Boston, MA  02111-1307, USA

include $(top_srcdir)/config/am_include.mk

noinst_HEADERS=../SoundTouch/AAFilter.h ../SoundTouch/cpu_detect.h ../SoundTouch/cpu_detect_x86.cpp ../SoundTouch/FIRFilter.h \
    ../SoundTouch/RateTransposer.h ../SoundTouch/TDStretch.h ../SoundTouch/PeakFinder.h ../SoundTouch/InterpolateCubic.h \
    ../SoundTouch/InterpolateLinear.h ../SoundTouch/InterpolateShannon.h

include_HEADERS=SoundTouchDLL.h

lib_LTLIBRARIES=libSoundTouchDll.la
#
libSoundTouchDll_la_SOURCES=../SoundTouch/AAFilter.cpp ../SoundTouch/FIRFilter.cpp \
    ../SoundTouch/FIFOSampleBuffer.cpp ../SoundTouch/RateTransposer.cpp ../SoundTouch/SoundTouch.cpp \
    ../SoundTouch/TDStretch.cpp ../SoundTouch/sse_optimized.cpp ../SoundTouch/cpu_detect_x86.cpp \
    ../SoundTouch/BPMDetect.cpp ../SoundTouch/PeakFinder.cpp ../SoundTouch/InterpolateLinear.cpp \
    ../SoundTouch/InterpolateCubic.cpp ../SoundTouch/InterpolateShannon.cpp SoundTouchDLL.cpp

# Compiler flags

# Modify the default 0.0.0 to LIB_SONAME.0.0
AM_LDFLAGS=$(LDFLAGS) -version-info @LIB_SONAME@

if X86
CXXFLAGS1=-mstackrealign -msse
endif

if X86_64
CXXFLAGS2=-fPIC
endif

AM_CXXFLAGS=$(CXXFLAGS) $(CXXFLAGS1) $(CXXFLAGS2) -shared -DDLL_EXPORTS -fvisibility=hidden
