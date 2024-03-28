/* Sonic library
   Copyright 2016
   Bill Cox
   This file is part of the Sonic Library.

   This file is licensed under the Apache 2.0 license.
*/

#ifdef  KISS_FFT
#include <stddef.h>  /* kiss_fft.h failes to load this */
#include <kiss_fft.h>
#include <kiss_fft_impl.h>
#else
#include <fftw3.h>
#endif
#include <float.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include "sonic.h"
#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif
#ifndef M_E
#define M_E 2.7182818284590452354
#endif

struct sonicSpectrumStruct;
typedef struct sonicSpectrumStruct* sonicSpectrum;

struct sonicSpectrogramStruct {
  sonicSpectrum* spectrums;
  double minPower, maxPower;
  int numSpectrums;
  int allocatedSpectrums;
  int sampleRate;
  int totalSamples;
};

struct sonicSpectrumStruct {
  sonicSpectrogram spectrogram;
  double* power;
  int numFreqs; /* Number of frequencies */
  int numSamples;
  int startingSample;
};

/* Print out spectrum data for debugging. */
static void dumpSpectrum(sonicSpectrum spectrum) {
  printf("spectrum numFreqs:%d numSamples:%d startingSample:%d\n",
         spectrum->numFreqs, spectrum->numSamples, spectrum->startingSample);
  printf("   ");
  int i;
  for (i = 0; i < spectrum->numFreqs; i++) {
    printf(" %.1f", spectrum->power[i]);
  }
  printf("\n");
}

/* Print out spectrogram data for debugging. */
void dumpSpectrogram(sonicSpectrogram spectrogram) {
  printf(
      "spectrogram minPower:%f maxPower:%f numSpectrums:%d totalSamples:%d\n",
      spectrogram->minPower, spectrogram->maxPower, spectrogram->numSpectrums,
      spectrogram->totalSamples);
  int i;
  for (i = 0; i < spectrogram->numSpectrums; i++) {
    dumpSpectrum(spectrogram->spectrums[i]);
  }
}

/* Create an new spectrum. */
static sonicSpectrum sonicCreateSpectrum(sonicSpectrogram spectrogram) {
  sonicSpectrum spectrum =
      (sonicSpectrum)calloc(1, sizeof(struct sonicSpectrumStruct));
  if (spectrum == NULL) {
    return NULL;
  }
  if (spectrogram->numSpectrums == spectrogram->allocatedSpectrums) {
    spectrogram->allocatedSpectrums <<= 1;
    spectrogram->spectrums = (sonicSpectrum*)realloc(
        spectrogram->spectrums,
        spectrogram->allocatedSpectrums * sizeof(sonicSpectrum));
    if (spectrogram->spectrums == NULL) {
      return NULL;
    }
  }
  spectrogram->spectrums[spectrogram->numSpectrums++] = spectrum;
  spectrum->spectrogram = spectrogram;
  return spectrum;
}

/* Destroy the spectrum. */
static void sonicDestroySpectrum(sonicSpectrum spectrum) {
  if (spectrum == NULL) {
    return;
  }
  if (spectrum->power != NULL) {
    free(spectrum->power);
  }
  free(spectrum);
}

/* Create an empty spectrogram. */
sonicSpectrogram sonicCreateSpectrogram(int sampleRate) {
  sonicSpectrogram spectrogram =
      (sonicSpectrogram)calloc(1, sizeof(struct sonicSpectrogramStruct));
  if (spectrogram == NULL) {
    return NULL;
  }
  spectrogram->allocatedSpectrums = 32;
  spectrogram->spectrums = (sonicSpectrum*)calloc(
      spectrogram->allocatedSpectrums, sizeof(sonicSpectrum));
  if (spectrogram->spectrums == NULL) {
    sonicDestroySpectrogram(spectrogram);
    return NULL;
  }
  spectrogram->sampleRate = sampleRate;
  spectrogram->minPower = DBL_MAX;
  spectrogram->maxPower = DBL_MIN;
  return spectrogram;
}

/* Destroy the spectrotram. */
void sonicDestroySpectrogram(sonicSpectrogram spectrogram) {
  if (spectrogram != NULL) {
    if (spectrogram->spectrums != NULL) {
      int i;
      for (i = 0; i < spectrogram->numSpectrums; i++) {
        sonicSpectrum spectrum = spectrogram->spectrums[i];
        sonicDestroySpectrum(spectrum);
      }
      free(spectrogram->spectrums);
    }
    free(spectrogram);
  }
}

/* Create a new bitmap.  This takes ownership of data. */
sonicBitmap sonicCreateBitmap(unsigned char* data, int numRows, int numCols) {
  sonicBitmap bitmap = (sonicBitmap)calloc(1, sizeof(struct sonicBitmapStruct));
  if (bitmap == NULL) {
    return NULL;
  }
  bitmap->data = data;
  bitmap->numRows = numRows;
  bitmap->numCols = numCols;
  return bitmap;
}

/* Destroy the bitmap. */
void sonicDestroyBitmap(sonicBitmap bitmap) {
  if (bitmap == NULL) {
    return;
  }
  if (bitmap->data != NULL) {
    free(bitmap->data);
  }
  free(bitmap);
}

/* Overlap-add the two pitch periods using a Hann window.  Caller must free the
 * result. */
static void computeOverlapAdd(short* samples, int period, int numChannels,
                              double* ola_samples) {
  int i;
  for (i = 0; i < period; i++) {
    double weight = (1.0 - cos(M_PI * i / period)) / 2.0;
    short sample1, sample2;
    if (numChannels == 1) {
      sample1 = samples[i];
      sample2 = samples[i + period];
    } else {
      /* Average the samples */
      int total1 = 0;
      int total2 = 0;
      int j;
      for (j = 0; j < numChannels; j++) {
        total1 += samples[i * numChannels + j];
        total2 += samples[(i + period) * numChannels + j];
      }
      sample1 = (total1 + (numChannels >> 1)) / numChannels;
      sample2 = (total2 + (numChannels >> 1)) / numChannels;
    }
    ola_samples[i] = weight * sample1 + (1.0 - weight) * sample2;
  }
}

#ifdef  KISS_FFT
/* Compute the amplitude of the kiss_complex number. */
static double magnitude(kiss_fft_cpx c) {
  return sqrt(c.r * c.r + c.i * c.i);
}
#else
/* Compute the amplitude of the fftw_complex number. */
static double magnitude(fftw_complex c) {
  return sqrt(c[0] * c[0] + c[1] * c[1]);
}
#endif

/* Add two pitch periods worth of samples to the spectrogram.  There must be
   2*period samples.  Time should advance one pitch period for each call to
   this function. */
void sonicAddPitchPeriodToSpectrogram(sonicSpectrogram spectrogram,
                                      short* samples, int numSamples,
                                      int numChannels) {
  int i;
  sonicSpectrum spectrum = sonicCreateSpectrum(spectrogram);
  spectrum->startingSample = spectrogram->totalSamples;
  spectrogram->totalSamples += numSamples;
  /* TODO: convert to fixed-point */
  double* in = calloc(numSamples, sizeof(double));
  int numFreqs = numSamples / 2 + 1;
  spectrum->numFreqs = numFreqs;
  spectrum->numSamples = numSamples;
  spectrum->power = (double*)calloc(spectrum->numFreqs, sizeof(double));
  computeOverlapAdd(samples, numSamples, numChannels, in);
#ifdef  KISS_FFT
  kiss_fft_cpx* cin = calloc(numFreqs, sizeof(kiss_fft_cpx));
  for (i=0; i<numFreqs; i++) {
    cin[i].r = in[i];
  }
  kiss_fft_cpx* out = calloc(numFreqs, sizeof(kiss_fft_cpx));
  kiss_fft_cfg kiss_plan = kiss_fft_alloc(numFreqs, 0, NULL, NULL);
  kiss_fft(kiss_plan, cin, out);
  free(cin);
#else
  fftw_complex* out = calloc(numFreqs, sizeof(fftw_complex));
  fftw_plan p = fftw_plan_dft_r2c_1d(numSamples, in, out, FFTW_ESTIMATE);
  fftw_execute(p);
  fftw_destroy_plan(p);
#endif  /* FFTW */
  /* Set the DC power to 0. */
  spectrum->power[0] = 0.0;
  for (i = 1; i < numFreqs; ++i) {
    double power = magnitude(out[i]) / numSamples;
    spectrum->power[i] = power;
    if (power > spectrogram->maxPower) {
      spectrogram->maxPower = power;
    }
    if (power < spectrogram->minPower) {
      spectrogram->minPower = power;
    }
  }
  free(in);
  free(out);
}

/* Linearly interpolate the power at a given position in the spectrogram. */
static double interpolateSpectrum(sonicSpectrum spectrum, int row,
                                  int numRows) {
  /* Flip the row so that we show lowest frequency on the bottom. */
  row = numRows - row - 1;
  /* We want the max row to be 1/2 the Niquist frequency, or 4 samples worth. */
  double spectrumFreqSpacing =
      (double)spectrum->spectrogram->sampleRate / spectrum->numSamples;
  double rowFreqSpacing = SONIC_MAX_SPECTRUM_FREQ / (numRows - 1);
  double targetFreq = row * rowFreqSpacing;
  int bottomIndex = targetFreq / spectrumFreqSpacing;
  double bottomPower = spectrum->power[bottomIndex];
  double topPower = spectrum->power[bottomIndex + 1];
  double position =
      (targetFreq - bottomIndex * spectrumFreqSpacing) / spectrumFreqSpacing;
  return (1.0 - position) * bottomPower + position * topPower;
}

/* Linearly interpolate the power at a given position in the spectrogram. */
static double interpolateSpectrogram(sonicSpectrum leftSpectrum,
                                     sonicSpectrum rightSpectrum, int row,
                                     int numRows, int colTime) {
  double leftPower = interpolateSpectrum(leftSpectrum, row, numRows);
  double rightPower = interpolateSpectrum(rightSpectrum, row, numRows);
  if (rightSpectrum->startingSample !=
      leftSpectrum->startingSample + leftSpectrum->numSamples) {
    fprintf(stderr, "Invalid sample spacing\n");
    exit(1);
  }
  int remainder = colTime - leftSpectrum->startingSample;
  double position = (double)remainder / leftSpectrum->numSamples;
  return (1.0 - position) * leftPower + position * rightPower;
}

/* Add one column of data to the output bitmap data. */
static void addBitmapCol(unsigned char* data, int col, int numCols, int numRows,
                         sonicSpectrogram spectrogram, sonicSpectrum spectrum,
                         sonicSpectrum nextSpectrum, int colTime) {
  double minPower = spectrogram->minPower;
  double maxPower = spectrogram->maxPower;
  int row;
  for (row = 0; row < numRows; row++) {
    double power =
        interpolateSpectrogram(spectrum, nextSpectrum, row, numRows, colTime);
    if (power < minPower && power > maxPower) {
      fprintf(stderr, "Power outside min/max range\n");
      exit(1);
    }
    double range = maxPower - minPower;
    /* Use log scale such that log(min) = 0, and log(max) = 255. */
    int value =
        256.0 * sqrt(sqrt(log((M_E - 1.0) * (power - minPower) / range + 1.0)));
    /* int value = (unsigned char)(((power - minPower)/range)*256); */
    if (value >= 256) {
      value = 255;
    }
    data[row * numCols + col] = 255 - value;
  }
}

/* Convert the spectrogram to a bitmap.  The returned array must be freed by
   the caller.  It will be rows*cols in size.  The pixels are written top row
   to bottom, and each row is left to right.  So, the pixel in the 5th row from
   the top, in the 18th column from the left in a 32x128 array would be in
   position 128*4 + 18.  NULL is returned if calloc fails to allocate the
   memory. */
sonicBitmap sonicConvertSpectrogramToBitmap(sonicSpectrogram spectrogram,
                                            int numRows, int numCols) {
  /* dumpSpectrogram(spectrogram); */
  unsigned char* data =
      (unsigned char*)calloc(numRows * numCols, sizeof(unsigned char));
  if (data == NULL) {
    return NULL;
  }
  int xSpectrum = 0; /* xSpectrum is index of nextSpectrum */
  sonicSpectrum spectrum = spectrogram->spectrums[xSpectrum++];
  sonicSpectrum nextSpectrum = spectrogram->spectrums[xSpectrum];
  int totalTime =
      spectrogram->spectrums[spectrogram->numSpectrums - 1]->startingSample;
  int col;
  for (col = 0; col < numCols; col++) {
    /* There must be at least two spectrums for this to work right. */
    double colTime = (double)totalTime * col / (numCols - 1);
    while (xSpectrum + 1 < spectrogram->numSpectrums &&
           colTime >= nextSpectrum->startingSample) {
      spectrum = nextSpectrum;
      nextSpectrum = spectrogram->spectrums[++xSpectrum];
    }
    addBitmapCol(data, col, numCols, numRows, spectrogram, spectrum,
                 nextSpectrum, colTime);
  }
  return sonicCreateBitmap(data, numRows, numCols);
}

/* Write a PGM image file, which is 8-bit grayscale and looks like:
    P2
    # CREATOR: libsonic
    640 400
    255
    ...
*/
int sonicWritePGM(sonicBitmap bitmap, char* fileName) {
  printf("Writing PGM to %s\n", fileName);
  FILE* file = fopen(fileName, "w");
  if (file == NULL) {
    return 0;
  }
  if (fprintf(file, "P2\n# CREATOR: libsonic\n%d %d\n255\n", bitmap->numCols,
              bitmap->numRows) < 0) {
    fclose(file);
    return 0;
  }
  int i;
  int numPixels = bitmap->numRows * bitmap->numCols;
  unsigned char* p = bitmap->data;
  for (i = 0; i < numPixels; i++) {
    if (fprintf(file, "%d\n", 255 - *p++) < 0) {
      fclose(file);
      return 0;
    }
  }
  fclose(file);
  return 1;
}
