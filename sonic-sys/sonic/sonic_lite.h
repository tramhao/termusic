/* Sonic library
   Copyright 2010
   Bill Cox
   This file is part of the Sonic Library.

   This file is licensed under the Apache 2.0 license.
*/

/*
  This is a stripped down version of sonic, to help it fit in micro-controllers.
  Only mono speedup remains.  All buffers are allocated statically.
*/

#ifdef __cplusplus
extern "C" {
#endif

/* Use a minimum pitch of 80 to reduce buffer sizes.  Set it back to 65 if you
   have the room in memory and find it sounds better. */
#define SONIC_MIN_PITCH 65
#define SONIC_MAX_PITCH 400

/* These are used to down-sample some inputs to improve speed */
#define SONIC_AMDF_FREQ 4000

/* This is the sample frequency.  You must hard-code it rather than passing it in. */
#define SONIC_SAMPLE_RATE 8000

/* This is the number of samples in the buffer size passed to Sonic.  */
#define SONIC_INPUT_SAMPLES 80

/* Initialize Sonic. */
void sonicInit(void);
/* Write input samples to the stream.  numSamples must be <= SONIC_INPUT_SAMPLES */
void sonicWriteShortToStream(short *samples, int numSamples);
/* Use this to read 16-bit data out of the stream.  Sometimes no data will
   be available, and zero is returned, which is not an error condition. */
int sonicReadShortFromStream(short *samples, int maxSamples);
/* Force the sonic stream to generate output using whatever data it currently
   has.  No extra delay will be added to the output, but flushing in the middle
   of words could introduce distortion. */
void sonicFlushStream(void);
/* Return the number of samples in the output buffer */
int sonicSamplesAvailable(void);
/* Set the speed of the stream. */
void sonicSetSpeed(float speed);
/* Set the scaling factor of the stream. */
void sonicSetVolume(float volume);

#ifdef __cplusplus
}
#endif
