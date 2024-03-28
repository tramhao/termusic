# This file was written by Bill Cox in 2010, and is licensed under the Apache
# 2.0 license.
#
# Note that -pthread is only included so that older Linux builds will be thread
# safe.  We call malloc, and older Linux versions only linked in the thread-safe
# malloc if -pthread is specified.

# Uncomment this if you want to link in spectrogram generation.  It is not
# needed to adjust speech speed or pitch.  It is included primarily to provide
# high-quality spectrograms with low CPU overhead, for applications such a
# speech recognition.
#USE_SPECTROGRAM=1

PREFIX=/usr

UNAME := $(shell uname)
ifeq ($(UNAME), Darwin)
  PREFIX=/usr/local
endif

BINDIR=$(PREFIX)/bin
LIBDIR=$(PREFIX)/lib
INCDIR=$(PREFIX)/include

SONAME=-soname,
SHARED_OPT=-shared
LIB_NAME=libsonic.so
LIB_INTERNAL_NAME=libsonic_internal.so
LIB_TAG=.0.3.0

ifeq ($(UNAME), Darwin)
  SONAME=-install_name,$(LIBDIR)/
  SHARED_OPT=-dynamiclib
  LIB_NAME=libsonic.dylib
  LIB_TAG=
endif

#CFLAGS=-Wall -Wno-unused-function -g -ansi -fPIC -pthread
CFLAGS ?= -O3
CFLAGS += -Wall -Wno-unused-function -ansi -fPIC -pthread

CC=gcc

# Set NO_MALLOC=1 as a parameter to make to compile Sonic with static buffers
# instead of calling malloc.  This is usefule primarily on microcontrollers.
ifeq ($(NO_MALLOC), 1)
  CFLAGS+= -DSONIC_NO_MALLOC
  # Set MAX_MEMORY=<memory size> if you need to incease the static memory buffer
  ifdef MAX_MEMORY
    CFLAGS+= -DSONIC_MAX_MEMORY=$(MAX_MEMORY)
  else
    CFLAGS+= -DSONIC_MAX_MEMORY=4096
  endif
endif

ifdef MIN_PITCH
  CFLAGS+= -DSONIC_MIN_PITCH=$(MIN_PITCH)
endif

EXTRA_SRC=
# Set this to empty if not using spectrograms.
FFTLIB=
ifeq ($(USE_SPECTROGRAM), 1)
  CFLAGS+= -DSONIC_SPECTROGRAM
  EXTRA_SRC+= spectrogram.c
  FFTLIB= -L$(LIBDIR) -lfftw3
endif
EXTRA_OBJ=$(EXTRA_SRC:.c=.o)

all: sonic sonic_lite $(LIB_NAME)$(LIB_TAG) libsonic.a libsonic_internal.a $(LIB_INTERNAL_NAME)$(LIB_TAG)

sonic: wave.o main.o libsonic.a
	$(CC) $(CFLAGS) $(LDFLAGS) -o sonic wave.o main.o libsonic.a -lm $(FFTLIB)

sonic_lite: wave.c main_lite.c sonic_lite.c sonic_lite.h
	$(CC) $(CFLAGS) $(LDFLAGS) -o sonic_lite sonic_lite.c wave.c main_lite.c

sonic.o: sonic.c sonic.h
	$(CC) $(CPPFLAGS) $(CFLAGS) -c sonic.c

# Define a version of sonic with the internal names defined so others (i.e. Speedy)
# can build new APIs that superscede the default API.
sonic_internal.o: sonic.c sonic.h
	$(CC) $(CPPFLAGS) $(CFLAGS) -DSONIC_INTERNAL -c sonic.c -o sonic_internal.o

wave.o: wave.c wave.h
	$(CC) $(CPPFLAGS) $(CFLAGS) -c wave.c

main.o: main.c sonic.h wave.h
	$(CC) $(CPPFLAGS) $(CFLAGS) -c main.c

spectrogram.o: spectrogram.c sonic.h
	$(CC) $(CPPFLAGS) $(CFLAGS) -DSONIC_SPECTROGRAM -c spectrogram.c

$(LIB_NAME)$(LIB_TAG): $(EXTRA_OBJ) sonic.o wave.o
	$(CC) $(CFLAGS) $(LDFLAGS) $(SHARED_OPT) -Wl,$(SONAME)$(LIB_NAME) $(EXTRA_OBJ) sonic.o -o $(LIB_NAME)$(LIB_TAG) $(FFTLIB) wave.o
ifneq ($(UNAME), Darwin)
	ln -sf $(LIB_NAME)$(LIB_TAG) $(LIB_NAME)
	ln -sf $(LIB_NAME)$(LIB_TAG) $(LIB_NAME).0
endif

$(LIB_INTERNAL_NAME)$(LIB_TAG): $(EXTRA_OBJ) sonic_internal.o wave.o  # No spectrogram needed here.
	$(CC) $(CFLAGS) $(LDFLAGS) $(SHARED_OPT) -Wl,$(SONAME)$(LIB_INTERNAL_NAME) $(EXTRA_OBJ) sonic_internal.o -o $(LIB_INTERNAL_NAME)$(LIB_TAG) $(FFTLIB)  wave.o
ifneq ($(UNAME), Darwin)
	ln -sf $(LIB_INTERNAL_NAME)$(LIB_TAG) $(LIB_INTERNAL_NAME)
	ln -sf $(LIB_INTERNAL_NAME)$(LIB_TAG) $(LIB_INTERNAL_NAME).0
endif

libsonic.a: $(EXTRA_OBJ) sonic.o wave.o
	$(AR) cqs libsonic.a $(EXTRA_OBJ) sonic.o wave.o

# Define a version of sonic with the internal names defined so others (i.e. Speedy)
# can build new APIs that superscede the default API.
libsonic_internal.a: $(EXTRA_OBJ) sonic_internal.o wave.o
	$(AR) cqs libsonic_internal.a $(EXTRA_OBJ) sonic_internal.o wave.o

install: sonic $(LIB_NAME)$(LIB_TAG) sonic.h
	install -d $(DESTDIR)$(BINDIR) $(DESTDIR)$(INCDIR) $(DESTDIR)$(LIBDIR)
	install sonic $(DESTDIR)$(BINDIR)
	install sonic.h $(DESTDIR)$(INCDIR)
	install libsonic.a $(DESTDIR)$(LIBDIR)
	install $(LIB_NAME)$(LIB_TAG) $(DESTDIR)$(LIBDIR)
ifneq ($(UNAME), Darwin)
	ln -sf $(LIB_NAME)$(LIB_TAG) $(DESTDIR)$(LIBDIR)/$(LIB_NAME)
	ln -sf $(LIB_NAME)$(LIB_TAG) $(DESTDIR)$(LIBDIR)/$(LIB_NAME).0
endif

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/sonic
	rm -f $(DESTDIR)$(INCDIR)/sonic.h
	rm -f $(DESTDIR)$(LIBDIR)/libsonic.a
	rm -f $(DESTDIR)$(LIBDIR)/$(LIB_NAME)$(LIB_TAG)
	rm -f $(DESTDIR)$(LIBDIR)/$(LIB_NAME).0
	rm -f $(DESTDIR)$(LIBDIR)/$(LIB_NAME)

clean:
	rm -f *.o sonic sonic_lite $(LIB_NAME)* libsonic.a libsonic_internal.a test.wav

check:
	./sonic -s 2.0 ./samples/talking.wav ./test.wav


libspeedy.so:
	cd speedy; make libspeedy.so  SONIC_DIR=.. FFTW_DIR=../../fftw

speedy_wave: libsonic_internal.so
	cd speedy; make speedy_wave SONIC_DIR=.. FFTW_DIR=../../fftw
	# You will probably also need to set the LDPATH.  For example
	#    export LD_LIBRARY_PATH=/usr/local/lib:../kissfft:speedy:.

