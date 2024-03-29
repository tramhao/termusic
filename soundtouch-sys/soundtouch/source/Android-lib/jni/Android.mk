LOCAL_PATH := $(call my-dir)

include $(CLEAR_VARS)

LOCAL_C_INCLUDES += $(LOCAL_PATH)/../../../include $(LOCAL_PATH)/../../SoundStretch
# *** Remember: Change -O0 into -O2 in add-applications.mk ***

LOCAL_MODULE    := soundtouch
LOCAL_SRC_FILES := soundtouch-jni.cpp ../../SoundTouch/AAFilter.cpp  ../../SoundTouch/FIFOSampleBuffer.cpp \
                ../../SoundTouch/FIRFilter.cpp ../../SoundTouch/cpu_detect_x86.cpp \
                ../../SoundTouch/sse_optimized.cpp ../../SoundStretch/WavFile.cpp \
                ../../SoundTouch/RateTransposer.cpp ../../SoundTouch/SoundTouch.cpp \
                ../../SoundTouch/InterpolateCubic.cpp ../../SoundTouch/InterpolateLinear.cpp \
                ../../SoundTouch/InterpolateShannon.cpp ../../SoundTouch/TDStretch.cpp \
                ../../SoundTouch/BPMDetect.cpp ../../SoundTouch/PeakFinder.cpp 

# for native audio
LOCAL_SHARED_LIBRARIES += -lgcc 
# --whole-archive -lgcc 
# for logging
LOCAL_LDLIBS    += -llog
# for native asset manager
#LOCAL_LDLIBS    += -landroid

# Custom Flags: 
# -fvisibility=hidden : don't export all symbols
LOCAL_CFLAGS += -fvisibility=hidden -fdata-sections -ffunction-sections -ffast-math

# OpenMP mode : enable these flags to enable using OpenMP for parallel computation 
#LOCAL_CFLAGS += -fopenmp
#LOCAL_LDFLAGS += -fopenmp


# Use ARM instruction set instead of Thumb for improved calculation performance in ARM CPUs	
LOCAL_ARM_MODE := arm

include $(BUILD_SHARED_LIBRARY)
