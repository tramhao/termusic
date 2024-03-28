/* Sonic library
   Copyright 2010
   Bill Cox
   This file is part of the Sonic Library.

   This file is licensed under the Apache 2.0 license.
*/

/* This file is designed for low-powered microcontrollers, minimizing memory
   compared to the fuller sonic.c implementation. */

#include "sonic_lite.h"

#include <string.h>

#define SONIC_MAX_PERIOD (SONIC_SAMPLE_RATE / SONIC_MIN_PITCH)
#define SONIC_MIN_PERIOD (SONIC_SAMPLE_RATE / SONIC_MAX_PITCH)
#define SONIC_SKIP (SONIC_SAMPLE_RATE / SONIC_AMDF_FREQ)
#define SONIC_INPUT_BUFFER_SIZE (2 * SONIC_MAX_PERIOD + SONIC_INPUT_SAMPLES)

struct sonicStruct {
  short inputBuffer[SONIC_INPUT_BUFFER_SIZE];
  short outputBuffer [2 * SONIC_MAX_PERIOD];
  short downSampleBuffer[(2 * SONIC_MAX_PERIOD) / SONIC_SKIP];
  float speed;
  float volume;
  int numInputSamples;
  int numOutputSamples;
  int remainingInputToCopy;
  int prevPeriod;
  int prevMinDiff;
};

static struct sonicStruct sonicStream;

/* Scale the samples by the factor.  Volume should be no greater than 127X, or
   it is possible to overflow the fixed-point mathi. */
static void scaleSamples(short *samples, int numSamples, float volume) {
  /* This is 24-bit integer and 8-bit fraction fixed-point representation. */
  int fixedPointVolume;
  int value;

  if (volume > 127.0) {
    volume = 127.0;
  }
  fixedPointVolume = volume * 256.0f;
  while (numSamples--) {
    value = (*samples * fixedPointVolume) >> 8;
    if (value > 32767) {
      value = 32767;
    } else if (value < -32767) {
      value = -32767;
    }
    *samples++ = value;
  }
}

/* Set the speed of the stream. */
void sonicSetSpeed(float speed) { sonicStream.speed = speed; }

/* Set the scaling factor of the stream. */
void sonicSetVolume(float volume) {
  sonicStream.volume = volume;
}

/* Create a sonic stream.  Return NULL only if we are out of memory and cannot
   allocate the stream. */
void sonicInit(void) {
  sonicStream.speed = 1.0;
  sonicStream.volume = 1.0f;
  sonicStream.numInputSamples = 0;;
  sonicStream.numOutputSamples = 0;
  sonicStream.remainingInputToCopy = 0;
  sonicStream.prevPeriod = 0;
  sonicStream.prevMinDiff = 0;
}

/* Add the input samples to the input buffer. */
static int addShortSamplesToInputBuffer(short *samples,
                                        int numSamples) {
  if (numSamples == 0) {
    return 1;
  }
  memcpy(sonicStream.inputBuffer + sonicStream.numInputSamples,
         samples, numSamples * sizeof(short));
  sonicStream.numInputSamples += numSamples;
  return 1;
}

/* Remove input samples that we have already processed. */
static void removeInputSamples(int position) {
  int remainingSamples = sonicStream.numInputSamples - position;

  if (remainingSamples > 0) {
    memmove(sonicStream.inputBuffer,
            sonicStream.inputBuffer + position,
            remainingSamples * sizeof(short));
  }
  sonicStream.numInputSamples = remainingSamples;
}

/* Just copy from the array to the output buffer */
static void copyToOutput(short *samples, int numSamples) {
  memcpy(sonicStream.outputBuffer + sonicStream.numOutputSamples,
         samples, numSamples * sizeof(short));
  sonicStream.numOutputSamples += numSamples;
}

/* Just copy from the input buffer to the output buffer. */
static int copyInputToOutput(int position) {
  int numSamples = sonicStream.remainingInputToCopy;

  if (numSamples > 2 * SONIC_MAX_PERIOD) {
    numSamples = 2 * SONIC_MAX_PERIOD;
  }
  copyToOutput(sonicStream.inputBuffer + position, numSamples);
  sonicStream.remainingInputToCopy -= numSamples;
  return numSamples;
}

/* Read short data out of the stream.  Sometimes no data will be available, and
   zero is returned, which is not an error condition. */
int sonicReadShortFromStream(short *samples, int maxSamples) {
  int numSamples = sonicStream.numOutputSamples;
  int remainingSamples = 0;

  if (numSamples == 0) {
    return 0;
  }
  if (numSamples > maxSamples) {
    remainingSamples = numSamples - maxSamples;
    numSamples = maxSamples;
  }
  memcpy(samples, sonicStream.outputBuffer, numSamples * sizeof(short));
  if (remainingSamples > 0) {
    memmove(sonicStream.outputBuffer, sonicStream.outputBuffer + numSamples,
            remainingSamples * sizeof(short));
  }
  sonicStream.numOutputSamples = remainingSamples;
  return numSamples;
}

/* Force the sonic stream to generate output using whatever data it currently
   has.  No extra delay will be added to the output, but flushing in the middle
   of words could introduce distortion. */
void sonicFlushStream(void) {
  int maxRequired = 2 * SONIC_MAX_PERIOD;
  int remainingSamples = sonicStream.numInputSamples;
  float speed = sonicStream.speed;
  int expectedOutputSamples = sonicStream.numOutputSamples + (int)((remainingSamples / speed) + 0.5f);

  memset(sonicStream.inputBuffer + remainingSamples, 0,
      sizeof(short) * (SONIC_INPUT_BUFFER_SIZE - remainingSamples));
  sonicStream.numInputSamples += 2 * maxRequired;
  sonicWriteShortToStream(NULL, 0);
  /* Throw away any extra samples we generated due to the silence we added */
  if (sonicStream.numOutputSamples > expectedOutputSamples) {
    sonicStream.numOutputSamples = expectedOutputSamples;
  }
  /* Empty input buffer */
  sonicStream.numInputSamples = 0;
  sonicStream.remainingInputToCopy = 0;
}

/* Return the number of samples in the output buffer */
int sonicSamplesAvailable(void) {
  return sonicStream.numOutputSamples;
}

/* If skip is greater than one, average skip samples together and write them to
   the down-sample buffer. */
static void downSampleInput(short *samples) {
  int numSamples = 2 * SONIC_MAX_PERIOD / SONIC_SKIP;
  int i, j;
  int value;
  short *downSamples = sonicStream.downSampleBuffer;

  for (i = 0; i < numSamples; i++) {
    value = 0;
    for (j = 0; j < SONIC_SKIP; j++) {
      value += *samples++;
    }
    value /= SONIC_SKIP;
    *downSamples++ = value;
  }
}

/* Find the best frequency match in the range, and given a sample skip multiple.
   For now, just find the pitch of the first channel. */
static int findPitchPeriodInRange(short *samples, int minPeriod, int maxPeriod,
                                  int* retMinDiff, int* retMaxDiff) {
  int period, bestPeriod = 0, worstPeriod = 255;
  short *s;
  short *p;
  short sVal, pVal;
  unsigned long diff, minDiff = 1, maxDiff = 0;
  int i;

  for (period = minPeriod; period <= maxPeriod; period++) {
    diff = 0;
    s = samples;
    p = samples + period;
    for (i = 0; i < period; i++) {
      sVal = *s++;
      pVal = *p++;
      diff += sVal >= pVal ? (unsigned short)(sVal - pVal)
                           : (unsigned short)(pVal - sVal);
    }
    /* Note that the highest number of samples we add into diff will be less
       than 256, since we skip samples.  Thus, diff is a 24 bit number, and
       we can safely multiply by numSamples without overflow */
    if (bestPeriod == 0 || diff * bestPeriod < minDiff * period) {
      minDiff = diff;
      bestPeriod = period;
    }
    if (diff * worstPeriod > maxDiff * period) {
      maxDiff = diff;
      worstPeriod = period;
    }
  }
  *retMinDiff = minDiff / bestPeriod;
  *retMaxDiff = maxDiff / worstPeriod;
  return bestPeriod;
}

/* At abrupt ends of voiced words, we can have pitch periods that are better
   approximated by the previous pitch period estimate.  Try to detect this case.  */
static int prevPeriodBetter(int minDiff, int maxDiff, int preferNewPeriod) {
  if (minDiff == 0 || sonicStream.prevPeriod == 0) {
    return 0;
  }
  if (preferNewPeriod) {
    if (maxDiff > minDiff * 3) {
      /* Got a reasonable match this period */
      return 0;
    }
    if (minDiff * 2 <= sonicStream.prevMinDiff * 3) {
      /* Mismatch is not that much greater this period */
      return 0;
    }
  } else {
    if (minDiff <= sonicStream.prevMinDiff) {
      return 0;
    }
  }
  return 1;
}

/* Find the pitch period.  This is a critical step, and we may have to try
   multiple ways to get a good answer.  This version uses Average Magnitude
   Difference Function (AMDF).  To improve speed, we down sample by an integer
   factor get in the 11KHz range, and then do it again with a narrower
   frequency range without down sampling */
static int findPitchPeriod(short *samples, int preferNewPeriod) {
  int minPeriod = SONIC_MIN_PERIOD;
  int maxPeriod = SONIC_MAX_PERIOD;
  int minDiff, maxDiff, retPeriod;
  int period;

  if (SONIC_SKIP == 1) {
    period = findPitchPeriodInRange(samples, minPeriod, maxPeriod, &minDiff, &maxDiff);
  } else {
    downSampleInput(samples);
    period = findPitchPeriodInRange(sonicStream.downSampleBuffer, minPeriod / SONIC_SKIP,
                                    maxPeriod / SONIC_SKIP, &minDiff, &maxDiff);
    period *= SONIC_SKIP;
    minPeriod = period - (SONIC_SKIP << 2);
    maxPeriod = period + (SONIC_SKIP << 2);
    if (minPeriod < SONIC_MIN_PERIOD) {
      minPeriod = SONIC_MIN_PERIOD;
    }
    if (maxPeriod > SONIC_MAX_PERIOD) {
      maxPeriod = SONIC_MAX_PERIOD;
    }
    period = findPitchPeriodInRange(samples, minPeriod, maxPeriod, &minDiff, &maxDiff);
  }
  if (prevPeriodBetter(minDiff, maxDiff, preferNewPeriod)) {
    retPeriod = sonicStream.prevPeriod;
  } else {
    retPeriod = period;
  }
  sonicStream.prevMinDiff = minDiff;
  sonicStream.prevPeriod = period;
  return retPeriod;
}

/* Overlap two sound segments, ramp the volume of one down, while ramping the
   other one from zero up, and add them, storing the result at the output. */
static void overlapAdd(int numSamples, short *out, short *rampDown, short *rampUp) {
  short *o;
  short *u;
  short *d;
  int t;

  o = out;
  u = rampUp;
  d = rampDown;
  for (t = 0; t < numSamples; t++) {
    *o = (*d * (numSamples - t) + *u * t) / numSamples;
    o++;
    d++;
    u++;
  }
}

/* Skip over a pitch period, and copy period/speed samples to the output */
static int skipPitchPeriod(short *samples, float speed, int period) {
  long newSamples;

  if (speed >= 2.0f) {
    newSamples = period / (speed - 1.0f);
  } else {
    newSamples = period;
    sonicStream.remainingInputToCopy = period * (2.0f - speed) / (speed - 1.0f);
  }
  overlapAdd(newSamples, sonicStream.outputBuffer + sonicStream.numOutputSamples,
      samples, samples + period);
  sonicStream.numOutputSamples += newSamples;
  return newSamples;
}

/* Resample as many pitch periods as we have buffered on the input. */
static void changeSpeed(float speed) {
  short *samples;
  int numSamples = sonicStream.numInputSamples;
  int position = 0, period, newSamples;
  int maxRequired = 2 * SONIC_MAX_PERIOD;

  /* printf("Changing speed to %f\n", speed); */
  if (sonicStream.numInputSamples < maxRequired) {
    return;
  }
  do {
    if (sonicStream.remainingInputToCopy > 0) {
      newSamples = copyInputToOutput(position);
      position += newSamples;
    } else {
      samples = sonicStream.inputBuffer + position;
      period = findPitchPeriod(samples, 1);
      newSamples = skipPitchPeriod(samples, speed, period);
      position += period + newSamples;
    }
  } while (position + maxRequired <= numSamples);
  removeInputSamples(position);
}

/* Resample as many pitch periods as we have buffered on the input.  Also scale
   the output by the volume. */
static void processStreamInput(void) {
  int originalNumOutputSamples = sonicStream.numOutputSamples;
  float speed = sonicStream.speed;

  if (speed > 1.00001) {
    changeSpeed(speed);
  } else {
    copyToOutput(sonicStream.inputBuffer, sonicStream.numInputSamples);
    sonicStream.numInputSamples = 0;
  }
  if (sonicStream.volume != 1.0f) {
    /* Adjust output volume. */
    scaleSamples( sonicStream.outputBuffer + originalNumOutputSamples,
        (sonicStream.numOutputSamples - originalNumOutputSamples), sonicStream.volume);
  }
}

/* Simple wrapper around sonicWriteFloatToStream that does the short to float
   conversion for you. */
void sonicWriteShortToStream(short *samples, int numSamples) {
  addShortSamplesToInputBuffer(samples, numSamples);
  processStreamInput();
}
