cmake_minimum_required(VERSION 3.1)
project(SoundTouch VERSION 2.3.3 LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 17)

include(GNUInstallDirs)

set(COMPILE_OPTIONS)

if(MSVC)
  set(COMPILE_DEFINITIONS /O2 /fp:fast)
else()
  list(APPEND COMPILE_OPTIONS -Wall -Wextra -Wzero-as-null-pointer-constant -Wno-unknown-pragmas)
  if(EMSCRIPTEN)
    list(APPEND COMPILE_OPTIONS -O3)
  else()
    list(APPEND COMPILE_OPTIONS -Ofast)
  endif()
endif()

#####################
# SoundTouch library

add_library(SoundTouch
  source/SoundTouch/AAFilter.cpp
  source/SoundTouch/BPMDetect.cpp
  source/SoundTouch/cpu_detect_x86.cpp
  source/SoundTouch/FIFOSampleBuffer.cpp
  source/SoundTouch/FIRFilter.cpp
  source/SoundTouch/InterpolateCubic.cpp
  source/SoundTouch/InterpolateLinear.cpp
  source/SoundTouch/InterpolateShannon.cpp
  source/SoundTouch/mmx_optimized.cpp
  source/SoundTouch/PeakFinder.cpp
  source/SoundTouch/RateTransposer.cpp
  source/SoundTouch/SoundTouch.cpp
  source/SoundTouch/sse_optimized.cpp
  source/SoundTouch/TDStretch.cpp
)
target_include_directories(SoundTouch PUBLIC
   $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
   $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}>
)

target_compile_definitions(SoundTouch PRIVATE ${COMPILE_DEFINITIONS})
target_compile_options(SoundTouch PRIVATE ${COMPILE_OPTIONS})

if(BUILD_SHARED_LIBS)
  set_target_properties(SoundTouch PROPERTIES
    VERSION ${CMAKE_PROJECT_VERSION}
  )
  if(WIN32)
    set_target_properties(SoundTouch PROPERTIES
      WINDOWS_EXPORT_ALL_SYMBOLS TRUE
    )
  else()
    set_target_properties(SoundTouch PROPERTIES
      SOVERSION ${PROJECT_VERSION_MAJOR}
    )
  endif()
endif()

option(INTEGER_SAMPLES "Use integers instead of floats for samples" OFF)
if(INTEGER_SAMPLES)
  target_compile_definitions(SoundTouch PRIVATE SOUNDTOUCH_INTEGER_SAMPLES)
else()
  target_compile_definitions(SoundTouch PRIVATE SOUNDTOUCH_FLOAT_SAMPLES)
endif()

if(CMAKE_SYSTEM_PROCESSOR MATCHES "^(armv7.*|armv8.*|aarch64.*)$")
  set(NEON_CPU ON)
else()
  set(NEON_CPU OFF)
endif()

option(NEON "Use ARM Neon SIMD instructions if in ARM CPU" ON)
if(${NEON} AND ${NEON_CPU})
  target_compile_definitions(SoundTouch PRIVATE SOUNDTOUCH_USE_NEON)
  if(NOT CMAKE_SYSTEM_PROCESSOR MATCHES "^aarch64.*$")
    target_compile_options(SoundTouch PRIVATE -mfpu=neon)
  endif()
endif()

find_package(OpenMP)
option(OPENMP "Use parallel multicore calculation through OpenMP" OFF)
if(OPENMP AND OPENMP_FOUND)
  set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} ${OpenMP_CXX_FLAGS}")
endif()

install(
  FILES
    include/BPMDetect.h
    include/FIFOSampleBuffer.h
    include/FIFOSamplePipe.h
    include/STTypes.h
    include/SoundTouch.h
    include/soundtouch_config.h
  DESTINATION
    "${CMAKE_INSTALL_INCLUDEDIR}/soundtouch"
)

install(TARGETS SoundTouch
  EXPORT SoundTouchTargets
  ARCHIVE DESTINATION "${CMAKE_INSTALL_LIBDIR}"
  LIBRARY DESTINATION "${CMAKE_INSTALL_LIBDIR}"
  RUNTIME DESTINATION "${CMAKE_INSTALL_BINDIR}"
  INCLUDES DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}"
)

#######################
# soundstretch utility

option(SOUNDSTRETCH "Build soundstretch command line utility." ON)
if(SOUNDSTRETCH)
  add_executable(soundstretch
    source/SoundStretch/main.cpp
    source/SoundStretch/RunParameters.cpp
    source/SoundStretch/WavFile.cpp
  )
  target_include_directories(soundstretch PUBLIC $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>)
  target_compile_definitions(soundstretch PRIVATE ${COMPILE_DEFINITIONS})
  target_compile_options(soundstretch PRIVATE ${COMPILE_OPTIONS})
  target_link_libraries(soundstretch PRIVATE SoundTouch)
  if(INTEGER_SAMPLES)
    target_compile_definitions(soundstretch PRIVATE SOUNDTOUCH_INTEGER_SAMPLES)
  endif()

  install(TARGETS soundstretch
    DESTINATION bin
  )
endif()

########################
# SoundTouchDll library

option(SOUNDTOUCH_DLL "Build SoundTouchDLL C wrapper library" OFF)
if(SOUNDTOUCH_DLL)
  add_library(SoundTouchDLL SHARED
    source/SoundTouchDLL/SoundTouchDLL.cpp
    source/SoundTouchDLL/SoundTouchDLL.rc
  )
  set_target_properties(SoundTouch PROPERTIES POSITION_INDEPENDENT_CODE TRUE)
  target_compile_options(SoundTouchDLL PRIVATE ${COMPILE_OPTIONS})
  set_target_properties(SoundTouchDLL PROPERTIES CXX_VISIBILITY_PRESET hidden)
  target_compile_definitions(SoundTouchDLL PRIVATE DLL_EXPORTS)
  target_include_directories(SoundTouchDLL PUBLIC $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>)
  target_link_libraries(SoundTouchDLL PRIVATE SoundTouch)
  install(FILES source/SoundTouchDLL/SoundTouchDLL.h DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}/soundtouch")
  install(TARGETS SoundTouchDLL EXPORT SoundTouchTargets)
endif()

########################

# pkgconfig
set(prefix "${CMAKE_INSTALL_PREFIX}")
set(execprefix "\${prefix}")
set(libdir "\${prefix}/${CMAKE_INSTALL_LIBDIR}")
set(includedir "\${prefix}/${CMAKE_INSTALL_INCLUDEDIR}")
set(VERSION "${CMAKE_PROJECT_VERSION}")
configure_file(soundtouch.pc.in "${CMAKE_CURRENT_BINARY_DIR}/soundtouch.pc" @ONLY)
install(FILES "${CMAKE_CURRENT_BINARY_DIR}/soundtouch.pc" DESTINATION "${CMAKE_INSTALL_LIBDIR}/pkgconfig")

# CMake config
include(CMakePackageConfigHelpers)
set(SOUNDTOUCH_INSTALL_CMAKEDIR "${CMAKE_INSTALL_LIBDIR}/cmake/SoundTouch")
install(
  EXPORT SoundTouchTargets
  FILE SoundTouchTargets.cmake
  NAMESPACE SoundTouch::
  DESTINATION "${SOUNDTOUCH_INSTALL_CMAKEDIR}"
)
configure_package_config_file(SoundTouchConfig.cmake.in
  "${CMAKE_CURRENT_BINARY_DIR}/SoundTouchConfig.cmake"
  INSTALL_DESTINATION "${SOUNDTOUCH_INSTALL_CMAKEDIR}"
)
write_basic_package_version_file(
  "${CMAKE_CURRENT_BINARY_DIR}/SoundTouchConfigVersion.cmake"
  VERSION "${CMAKE_PROJECT_VERSION}"
  COMPATIBILITY SameMajorVersion
)
install(
  FILES
    "${CMAKE_CURRENT_BINARY_DIR}/SoundTouchConfig.cmake"
    "${CMAKE_CURRENT_BINARY_DIR}/SoundTouchConfigVersion.cmake"
  DESTINATION "${SOUNDTOUCH_INSTALL_CMAKEDIR}"
)
