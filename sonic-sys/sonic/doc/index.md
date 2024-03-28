# libsonic Home Page

[Download the latest tar-ball from here](download).

The source code repository can be cloned using git:

    $ git clone git://github.com/waywardgeek/sonic.git

The source code for the Android version, sonic-ndk, can be cloned with:

    $ git clone git://github.com/waywardgeek/sonic-ndk.git

There is a simple test app for android that demos capabilities.  You can
[install the Android application from here](Sonic-NDK.apk)

There is a new native Java port, which is very fast!  Checkout Sonic.java and
Main.java in the latest tar-ball, or get the code from git.

## Overview

Sonic is free software for speeding up or slowing down speech.  While similar to
other algorithms that came before, Sonic is optimized for speed ups of over 2X.
There is a simple sonic library in ANSI C, and one in pure Java.  Both are
designed to easily be integrated into streaming voice applications, like TTS
back ends.  While a very new project, it is already integrated into:

- espeak
- Debian Sid as package libsonic
- Android Astro Player Nova
- Android Osplayer
- Multiple closed source TTS engines

The primary motivation behind sonic is to enable the blind and visually impaired
to improve their productivity with free software speech engines, like espeak.
Sonic can also be used by the sighted.  For example, sonic can improve the
experience of listening to an audio book on an Android phone.

Sonic is Copyright 2010, 2011, Bill Cox, all rights reserved.  It is released
as under the Apache 2.0 license.  Feel free to contact me at
<waywardgeek@gmail.com>.  One user was concerned about patents.  I believe the
sonic algorithms do not violate any patents, as most of it is very old, based
on [PICOLA](https://web.archive.org/web/20120731100136/http://keizai.yokkaichi-u.ac.jp/~ikeda/research/picola.html), and
the new part, for greater than 2X speed up, is clearly a capability most
developers ignore, and would not bother to patent.

## Comparison to Other Solutions

In short, Sonic is better for speech, while WSOLA is better for music.

A popular alternative is SoundTouch.  SoundTouch uses WSOLA, an algorithm
optimized for changing the tempo of music.  No WSOLA based program performs well
for speech (contrary to the inventor's estimate of WSOLA).  Listen to [this
soundstretch sample](soundstretch.wav), which uses SoundTouch, and compare
it to [this sonic sample](sonic.wav).  Both are sped up by 2X.  WSOLA
introduces unacceptable levels of distortion, making speech impossible to
understand at high speed (over 2.5X) by blind speed listeners.

However, there are decent free software algorithms for speeding up speech.  They
are all in the TD-PSOLA family.  For speech rates below 2X, sonic uses PICOLA,
which I find to be the best algorithm available.  A slightly buggy
implementation of PICOLA is available in the spandsp library.  I find the one in
RockBox quite good, though it's limited to 2X speed up.  So far as I know, only
sonic is optimized for speed factors needed by the blind, up to 6X.

Sonic does all of its CPU intensive work with integer math, and works well on
ARM CPUs without FPUs.  It supports multiple channels (stereo), and is also able
to change the pitch of a voice.  It works well in streaming audio applications,
and can deal with sound streams in 16-bit signed integer, 32-bit floating point,
or 8-bit unsigned formats.  The source code is in plain ANSI C.  In short, it's
production ready.

## Using libsonic in your program

Sonic is still a new library, but is in Debian Sid.  It will take a while
for it to filter out into all the other distros.  For now, feel free to simply
add sonic.c and sonic.h to your application (or Sonic.java), but consider
switching to -lsonic once the library is available on your distro.

The file [main.c](main.c) is the source code for the sonic command-line application.  It
is meant to be useful as example code.  Feel free to copy directly from main.c
into your application, as main.c is in the public domain.  Dependencies listed
in debian/control like libsndfile are there to compile the sonic command-line
application.  Libsonic has no external dependencies.

There are basically two ways to use sonic: batch or stream mode.  The simplest
is batch mode where you pass an entire sound sample to sonic.  All you do is
call one function, like this:

    sonicChangeShortSpeed(samples, numSamples, speed, pitch, rate, volume, useChordPitch, sampleRate, numChannels);

This will change the speed and pitch of the sound samples pointed to by samples,
which should be 16-bit signed integers.  Stereo mode is supported, as
is any arbitrary number of channels.  Samples for each channel should be
adjacent in the input array.  Because the samples are modified in-place, be sure
that there is room in the samples array for the speed-changed samples.  In
general, if you are speeding up, rather than slowing down, it will be safe to
have no extra padding.  If your sound samples are mono, and you don't want to
scale volume or playback rate, and if you want normal pitch scaling, then call
it like this:

    sonicChangeShortSpeed(samples, numSamples, speed, pitch, 1.0f, 1.0f, 0, sampleRate, 1);

The other way to use libsonic is in stream mode.  This is more complex, but
allows sonic to be inserted into a sound stream with fairly low latency.  The
current maximum latency in sonic is 31 milliseconds, which is enough to process
two pitch periods of voice as low as 65 Hz.  In general, the latency is equal to
two pitch periods, which is typically closer to 20 milliseconds.

To process a sound stream, you must create a sonicStream object, which contains
all of the state used by sonic.  Sonic should be thread safe, and multiple
sonicStream objects can be used at the same time.  You create a sonicStream
object like this:

    sonicStream stream = sonicCreateStream(sampleRate, numChannels);

When you're done with a sonic stream, you can free it's memory with:

    sonicDestroyStream(stream);

By default, a sonic stream sets the speed, pitch, rate, and volume to 1.0, which means
no change at all to the sound stream.  Sonic detects this case, and simply
copies the input to the output to reduce CPU load.  To change the speed, pitch,
rate, or volume, set the parameters using:

    sonicSetSpeed(stream, speed);
    sonicSetPitch(stream, pitch);
    sonicSetRate(stream, rate);
    sonicSetVolume(stream, volume);

These four parameters are floating point numbers.  A speed of 2.0 means to
double speed of speech.  A pitch of 0.95 means to lower the pitch by about 5%,
and a volume of 1.4 means to multiply the sound samples by 1.4, clipping if we
exceed the maximum range of a 16-bit integer.  Speech rate scales how fast
speech is played.  A 2.0 value will make you sound like a chipmunk talking very
fast.  A 0.7 value will make you sound like a giant talking slowly.

By default, pitch is modified by changing the rate, and then using speed
modification to bring the speed back to normal.  This allows for a wide range of
pitch changes, but changing the pitch makes the speaker sound larger or smaller,
too.  If you want to make the person sound like the same person, but talking at
a higher or lower pitch, then enable the vocal chord emulation mode for pitch
scaling, using:

    sonicSetChordPitch(stream, 1);

However, only small changes to pitch should be used in this mode, as it
introduces significant distortion otherwise.

After setting the sound parameters, you write to the stream like this:

    sonicWriteShortToStream(stream, samples, numSamples);

You read the sped up speech samples from sonic like this:

    samplesRead = sonicReadShortFromStream(stream, outBuffer, maxBufferSize);
    if(samplesRead > 0) {
	/* Do something with the output samples in outBuffer, like send them to
	 * the sound device. */
    }

You may change the speed, pitch, rate, and volume parameters at any time, without
having to flush or create a new sonic stream.

When your sound stream ends, there may be several milliseconds of sound data in
the sonic stream's buffers.  To force sonic to process those samples use:

    sonicFlushStream(stream);

Then, read those samples as above.  That's about all there is to using libsonic.
There are some more functions as a convenience for the user, like
sonicGetSpeed.  Other sound data formats are supported: signed char and float.
If float, the sound data should be between -1.0 and 1.0.  Internally, all sound
data is converted to 16-bit integers for processing.
