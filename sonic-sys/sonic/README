Sonic is a simple algorithm for speeding up or slowing down speech.  However,
it's optimized for speed ups of over 2X, unlike previous algorithms for changing
speech rate.  The Sonic library is a very simple ANSI C library that is designed
to easily be integrated into streaming voice applications, like TTS back ends.

The primary motivation behind Sonic is to enable the blind and visually impaired
to improve their productivity with open source speech engines, like espeak.
Sonic can also be used by the sighted.  For example, Sonic can improve the
experience of listening to an audio book on an Android phone.

A native Java port of Sonic is in Sonic.java.  Main.java is a simple example of
how to use Sonic.java.  To play with it, you'll need a "talking.wav" file in the
current directory, and you'll want to change the speed, pitch or other
parameters manually in Main.java, in the main method.

Sonic is Copyright 2010, 2011, Bill Cox, all rights reserved.  It is released
under the Apache 2.0 license, to promote usage as widely as possible.

Performance test:

I sped up a 751958176 byte wav file with sonic (a 9 hour, 28 minute mono audio
file encoded at 16-bit 11.KHz), but with the output writing disabled.  The
reported time, running Ubuntu 11.04 on my HP Pavilion dm4 laptop was:

real    0m50.839s
user    0m47.370s
sys     0m0.620s

The Java version is not much slower.  It reported:

real    0m52.043s
user    0m51.190s
sys     0m0.310s

Update, May 7, 2017
-------------------
I upgraded the pitch change algorithm to use a 12-point sinc FIR filter for
interpolation, rather than linearly interpolating between points.  This
significantly reduces noise introduced by the pitch change algorithm.  It is
most noticable in low-sample-rate streams, such as the 11,025 Hz output of the
Eloquence TTS engine.  The upgrade is in both the C and Java versions.


Author: Bill Cox
email: waywardgeek@gmail.com
