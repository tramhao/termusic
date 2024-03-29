#!/bin/bash
#
# This script is deprecated. Don't use this, the makefile can now compile
# the dynamic-link library 'libSoundTouchDLL.so' automatically.
#
# This script compiles SoundTouch dynamic-link library for GNU environment
# with wrapper functions that are easier to import to Java / Mono / etc
#

arch=$(uname -m)
flags=""

if [[ $arch == *"86"* ]]; then
    # Intel x86/x64 architecture
    flags="$flags -mstackrealign -msse"

    if [[ $arch == *"_64" ]]; then
        flags="$flags -fPIC"
    fi
fi

echo "*************************************************************************"
echo "NOTE: Rather use the makefile that can now build the dynamic-link library"
echo "*************************************************************************"
echo ""
echo "Building SoundTouchDLL for $arch with flags:$flags"

g++ -O3 -ffast-math -shared $flags -DDLL_EXPORTS -fvisibility=hidden -I../../include \
    -I../SoundTouch -o SoundTouchDll.so SoundTouchDLL.cpp ../SoundTouch/*.cpp
