/* This file was written by Bill Cox in 2010, and is licensed under the Apache
   2.0 license.

   This file is meant as a simple example for how to use libsonic.  It is also a
   useful utility on its own, which can speed up or slow down wav files, change
   pitch, and scale volume. */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "sonic.h"
#include "wave.h"

#define BUFFER_SIZE 2048

/* Run sonic. */
static void runSonic(char* inFileName, char* outFileName, float speed,
                     float pitch, float rate, float volume, int outputSampleRate,
                     int emulateChordPitch, int quality, int computeSpectrogram,
                     int numRows, int numCols) {
  waveFile inFile, outFile = NULL;
  sonicStream stream;
  short inBuffer[BUFFER_SIZE], outBuffer[BUFFER_SIZE];
  int sampleRate, numChannels, samplesRead, samplesWritten;

  inFile = openInputWaveFile(inFileName, &sampleRate, &numChannels);
  if (outputSampleRate != 0) {
    sampleRate = outputSampleRate;
  }
  if (inFile == NULL) {
    fprintf(stderr, "Unable to read wave file %s\n", inFileName);
    exit(1);
  }
  if (!computeSpectrogram) {
    outFile = openOutputWaveFile(outFileName, sampleRate, numChannels);
    if (outFile == NULL) {
      closeWaveFile(inFile);
      fprintf(stderr, "Unable to open wave file %s for writing\n", outFileName);
      exit(1);
    }
  }
  stream = sonicCreateStream(sampleRate, numChannels);
  sonicSetSpeed(stream, speed);
  sonicSetPitch(stream, pitch);
  sonicSetRate(stream, rate);
  sonicSetVolume(stream, volume);
  sonicSetChordPitch(stream, emulateChordPitch);
  sonicSetQuality(stream, quality);
#ifdef SONIC_SPECTROGRAM
  if (computeSpectrogram) {
    sonicComputeSpectrogram(stream);
  }
#endif  /* SONIC_SPECTROGRAM */
  do {
    samplesRead = readFromWaveFile(inFile, inBuffer, BUFFER_SIZE / numChannels);
    if (samplesRead == 0) {
      sonicFlushStream(stream);
    } else {
      sonicWriteShortToStream(stream, inBuffer, samplesRead);
    }
    if (!computeSpectrogram) {
      do {
        samplesWritten = sonicReadShortFromStream(stream, outBuffer,
                                                  BUFFER_SIZE / numChannels);
        if (samplesWritten > 0 && !computeSpectrogram) {
          writeToWaveFile(outFile, outBuffer, samplesWritten);
        }
      } while (samplesWritten > 0);
    }
  } while (samplesRead > 0);
#ifdef SONIC_SPECTROGRAM
  if (computeSpectrogram) {
    sonicSpectrogram spectrogram = sonicGetSpectrogram(stream);
    sonicBitmap bitmap =
        sonicConvertSpectrogramToBitmap(spectrogram, numRows, numCols);
    sonicWritePGM(bitmap, outFileName);
    sonicDestroyBitmap(bitmap);
  }
#endif  /* SONIC_SPECTROGRAM */
  sonicDestroyStream(stream);
  closeWaveFile(inFile);
  if (!computeSpectrogram) {
    closeWaveFile(outFile);
  }
}

/* Print the usage. */
static void usage(void) {
  fprintf(
      stderr,
      "Usage: sonic [OPTION]... infile outfile\n"
      "    -c         -- Modify pitch by emulating vocal chords vibrating\n"
      "                  faster or slower.\n"
      "    -o         -- Override the sample rate of the output.  -o 44200\n"
      "                  on an input file at 22100 KHz will play twice as fast\n"
      "                  and have twice the pitch.\n"
      "    -p pitch   -- Set pitch scaling factor.  1.3 means 30%% higher.\n"
      "    -q         -- Disable speed-up heuristics.  May increase quality.\n"
      "    -r rate    -- Set playback rate.  2.0 means 2X faster, and 2X "
      "pitch.\n"
      "    -s speed   -- Set speed up factor.  2.0 means 2X faster.\n"
#ifdef SONIC_SPECTROGRAM
      "    -S width height -- Write a spectrogram in outfile in PGM format.\n"
#endif  /* SONIC_SPECTROGRAM */
      "    -v volume  -- Scale volume by a constant factor.\n");
  exit(1);
}

int main(int argc, char** argv) {
  char* inFileName;
  char* outFileName;
  float speed = 1.0f;
  float pitch = 1.0f;
  float rate = 1.0f;
  float volume = 1.0f;
  int outputSampleRate = 0;  /* Means use the input file sample rate. */
  int emulateChordPitch = 0;
  int quality = 0;
  int xArg = 1;
  int computeSpectrogram = 0;
  int numRows = 0, numCols = 0;

  while (xArg < argc && *(argv[xArg]) == '-') {
    if (!strcmp(argv[xArg], "-c")) {
      emulateChordPitch = 1;
      printf("Scaling pitch linearly.\n");
    } else if (!strcmp(argv[xArg], "-o")) {
      xArg++;
      if (xArg < argc) {
        outputSampleRate = atoi(argv[xArg]);
        printf("Setting output sample rate to %d\n", outputSampleRate);
      }
    } else if (!strcmp(argv[xArg], "-p")) {
      xArg++;
      if (xArg < argc) {
        pitch = atof(argv[xArg]);
        printf("Setting pitch to %0.2fX\n", pitch);
      }
    } else if (!strcmp(argv[xArg], "-q")) {
      quality = 1;
      printf("Disabling speed-up heuristics\n");
    } else if (!strcmp(argv[xArg], "-r")) {
      xArg++;
      if (xArg < argc) {
        rate = atof(argv[xArg]);
        if (rate == 0.0f) {
          usage();
        }
        printf("Setting rate to %0.2fX\n", rate);
      }
    } else if (!strcmp(argv[xArg], "-s")) {
      xArg++;
      if (xArg < argc) {
        speed = atof(argv[xArg]);
        printf("Setting speed to %0.2fX\n", speed);
      }
#ifdef SONIC_SPECTROGRAM
    } else if (!strcmp(argv[xArg], "-S")) {
      xArg++;
      if (xArg < argc) {
        numCols = atof(argv[xArg]);
      }
      xArg++;
      if (xArg < argc) {
        numRows = atof(argv[xArg]);
        computeSpectrogram = 1;
        printf("Computing spectrogram %d wide and %d tall\n", numCols, numRows);
      }
#endif  /* SONIC_SPECTROGRAM */
    } else if (!strcmp(argv[xArg], "-v")) {
      xArg++;
      if (xArg < argc) {
        volume = atof(argv[xArg]);
        printf("Setting volume to %0.2f\n", volume);
      }
    }
    xArg++;
  }
  if (argc - xArg != 2) {
    usage();
  }
  inFileName = argv[xArg];
  outFileName = argv[xArg + 1];
  runSonic(inFileName, outFileName, speed, pitch, rate, volume,
           outputSampleRate, emulateChordPitch, quality,
           computeSpectrogram, numRows, numCols);
  return 0;
}
