/* Sonic library
   Copyright 2010
   Bill Cox
   This file is part of the Sonic Library.

   This file is licensed under the Apache 2.0 license.
*/

/*
This file supports read/write wave files.
*/
#include "wave.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define WAVE_BUF_LEN 4096

struct waveFileStruct {
  int numChannels;
  int sampleRate;
  FILE* soundFile;
  int bytesWritten; /* The number of bytes written so far, including header */
  int failed;
  int isInput;
};

/* Write a string to a file. */
static void writeBytes(waveFile file, void* bytes, int length) {
  size_t bytesWritten;

  if (file->failed) {
    return;
  }
  bytesWritten = fwrite(bytes, sizeof(char), length, file->soundFile);
  if (bytesWritten != length) {
    fprintf(stderr, "Unable to write to output file");
    file->failed = 1;
  }
  file->bytesWritten += bytesWritten;
}

/* Write a string to a file. */
static void writeString(waveFile file, char* string) {
  writeBytes(file, string, strlen(string));
}

/* Write an integer to a file in little endian order. */
static void writeInt(waveFile file, int value) {
  char bytes[4];
  int i;

  for (i = 0; i < 4; i++) {
    bytes[i] = value;
    value >>= 8;
  }
  writeBytes(file, bytes, 4);
}

/* Write a short integer to a file in little endian order. */
static void writeShort(waveFile file, short value) {
  char bytes[2];
  int i;

  for (i = 0; i < 2; i++) {
    bytes[i] = value;
    value >>= 8;
  }
  writeBytes(file, bytes, 2);
}

/* Read bytes from the input file. Return the number of bytes actually read. */
static int readBytes(waveFile file, void* bytes, int length) {
  if (file->failed) {
    return 0;
  }
  return fread(bytes, sizeof(char), length, file->soundFile);
}

/* Read an exact number of bytes from the input file. */
static void readExactBytes(waveFile file, void* bytes, int length) {
  int numRead;

  if (file->failed) {
    return;
  }
  numRead = fread(bytes, sizeof(char), length, file->soundFile);
  if (numRead != length) {
    fprintf(stderr, "Failed to read requested bytes from input file\n");
    file->failed = 1;
  }
}

/* Read an integer from the input file */
static int readInt(waveFile file) {
  unsigned char bytes[4];
  int value = 0, i;

  readExactBytes(file, bytes, 4);
  for (i = 3; i >= 0; i--) {
    value <<= 8;
    value |= bytes[i];
  }
  return value;
}

/* Read a short from the input file */
static int readShort(waveFile file) {
  unsigned char bytes[2];
  int value = 0, i;

  readExactBytes(file, bytes, 2);
  for (i = 1; i >= 0; i--) {
    value <<= 8;
    value |= bytes[i];
  }
  return value;
}

/* Read a string from the input and compare it to an expected string. */
static void expectString(waveFile file, char* expectedString) {
  char buf[11]; /* Be sure that we never call with a longer string */
  int length = strlen(expectedString);

  if (length > 10) {
    fprintf(stderr, "Internal error: expected string too long\n");
    file->failed = 1;
  } else {
    readExactBytes(file, buf, length);
    buf[length] = '\0';
    if (strcmp(expectedString, buf)) {
      fprintf(stderr, "Unsupported wave file format: Expected '%s', got '%s;\n",
          expectedString, buf);
      file->failed = 1;
    }
  }
}

/* Write the header of the wave file. */
static void writeHeader(waveFile file, int sampleRate, int numChannels) {
  /* write the wav file per the wav file format */
  writeString(file, "RIFF"); /* 00 - RIFF */
  /* We have to fseek and overwrite this later when we close the file because */
  /* we don't know how big it is until then. */
  writeInt(file,
           36 /* + dataLength */); /* 04 - how big is the rest of this file? */
  writeString(file, "WAVE");       /* 08 - WAVE */
  writeString(file, "fmt ");       /* 12 - fmt */
  writeInt(file, 16);              /* 16 - size of this chunk */
  writeShort(
      file,
      1); /* 20 - what is the audio format? 1 for PCM = Pulse Code Modulation */
  writeShort(file,
             numChannels);    /* 22 - mono or stereo? 1 or 2?  (or 5 or ???) */
  writeInt(file, sampleRate); /* 24 - samples per second (numbers per second) */
  writeInt(file, sampleRate * 2); /* 28 - bytes per second */
  writeShort(file, 2); /* 32 - # of bytes in one sample, for all channels */
  writeShort(
      file, 16); /* 34 - how many bits in a sample(number)?  usually 16 or 24 */
  writeString(file, "data"); /* 36 - data */
  writeInt(file, 0);         /* 40 - how big is this data chunk */
}

/* Read the header of the wave file. */
static int readHeader(waveFile file) {
  int data;

  expectString(file, "RIFF");
  data = readInt(file);          /* 04 - how big is the rest of this file? */
  expectString(file, "WAVE");    /* 08 - WAVE */
  expectString(file, "fmt ");    /* 12 - fmt */
  int chunkSize = readInt(file); /* 16 or 18 - size of this chunk */
  if (chunkSize != 16 && chunkSize != 18) {
    fprintf(stderr, "Only basic wave files are supported\n");
    return 0;
  }
  data = readShort(file); /* 20 - what is the audio format? 1 for PCM = Pulse
                             Code Modulation */
  if (data != 1) {
    fprintf(stderr, "Only PCM wave files are supported (not %d)\n", data);
    return 0;
  }
  file->numChannels =
      readShort(file); /* 22 - mono or stereo? 1 or 2?  (or 5 or ???) */
  file->sampleRate =
      readInt(file); /* 24 - samples per second (numbers per second) */
  readInt(file);     /* 28 - bytes per second */
  readShort(file);   /* 32 - # of bytes in one sample, for all channels */
  data = readShort(
      file); /* 34 - how many bits in a sample(number)?  usually 16 or 24 */
  if (data != 16) {
    fprintf(stderr, "Only 16 bit PCM wave files are supported\n");
    return 0;
  }
  if (chunkSize == 18) { /* ffmpeg writes 18, and so has 2 extra bytes here */
    data = readShort(file);
  }

  /* Read and discard chunks until we find the "data" chunk or fail */
  char chunk[5];
  chunk[4] = 0;

  while (1) {
    readExactBytes(file, chunk, 4);  /* chunk id */
    int size = readInt(file);        /* how big is this data chunk */
    if (strcmp(chunk, "data") == 0) {
      return 1;
    }
    if (fseek(file->soundFile, size, SEEK_CUR) != 0) {
      fprintf(stderr, "Failed to seek on input file.\n");
      return 0;
    }
  }
}

/* Close the input or output file and free the waveFile. */
static void closeFile(waveFile file) {
  FILE* soundFile = file->soundFile;

  if (soundFile != NULL) {
    fclose(soundFile);
    file->soundFile = NULL;
  }
  free(file);
}

/* Open a 16-bit little-endian wav file for reading.  It may be mono or stereo.
 */
waveFile openInputWaveFile(const char* fileName, int* sampleRate, int* numChannels) {
  waveFile file;
  FILE* soundFile = fopen(fileName, "rb");

  if (soundFile == NULL) {
    fprintf(stderr, "Unable to open wave file %s for reading\n", fileName);
    return NULL;
  }
  file = (waveFile)calloc(1, sizeof(struct waveFileStruct));
  file->soundFile = soundFile;
  file->isInput = 1;
  if (!readHeader(file)) {
    closeFile(file);
    return NULL;
  }
  *sampleRate = file->sampleRate;
  *numChannels = file->numChannels;
  return file;
}

/* Open a 16-bit little-endian wav file for writing.  It may be mono or stereo.
 */
waveFile openOutputWaveFile(const char* fileName, int sampleRate, int numChannels) {
  waveFile file;
  FILE* soundFile = fopen(fileName, "wb");

  if (soundFile == NULL) {
    fprintf(stderr, "Unable to open wave file %s for writing\n", fileName);
    return NULL;
  }
  file = (waveFile)calloc(1, sizeof(struct waveFileStruct));
  file->soundFile = soundFile;
  file->sampleRate = sampleRate;
  file->numChannels = numChannels;
  writeHeader(file, sampleRate, numChannels);
  if (file->failed) {
    closeFile(file);
    return NULL;
  }
  return file;
}

/* Close the sound file. */
int closeWaveFile(waveFile file) {
  FILE* soundFile = file->soundFile;
  int passed = 1;

  if (!file->isInput) {
    if (fseek(soundFile, 4, SEEK_SET) != 0) {
      fprintf(stderr, "Failed to seek on input file.\n");
      passed = 0;
    } else {
      /* Now update the file to have the correct size. */
      writeInt(file, file->bytesWritten - 8);
      if (file->failed) {
        fprintf(stderr, "Failed to write wave file size.\n");
        passed = 0;
      }
      if (fseek(soundFile, 40, SEEK_SET) != 0) {
        fprintf(stderr, "Failed to seek on input file.\n");
        passed = 0;
      } else {
        /* Now update the file to have the correct size. */
        writeInt(file, file->bytesWritten - 48);
        if (file->failed) {
          fprintf(stderr, "Failed to write wave file size.\n");
          passed = 0;
        }
      }
    }
  }
  closeFile(file);
  return passed;
}

/* Read from the wave file.  Return the number of samples read.
   numSamples and maxSamples are the number of **multi-channel** samples */
int readFromWaveFile(waveFile file, short* buffer, int maxSamples) {
  int i, bytesRead, samplesRead;
  int bytePos = 0;
  unsigned char bytes[WAVE_BUF_LEN];
  short sample;

  if (maxSamples * file->numChannels * 2 > WAVE_BUF_LEN) {
    maxSamples = WAVE_BUF_LEN / (file->numChannels * 2);
  }
  bytesRead = readBytes(file, bytes, maxSamples * file->numChannels * 2);
  samplesRead = bytesRead / (file->numChannels * 2);
  for (i = 0; i < samplesRead * file->numChannels; i++) {
    sample = bytes[bytePos++];
    sample |= (unsigned int)bytes[bytePos++] << 8;
    *buffer++ = sample;
  }
  return samplesRead;
}

/* Write to the wave file. */
int writeToWaveFile(waveFile file, short* buffer, int numSamples) {
  int i;
  int bytePos = 0;
  unsigned char bytes[WAVE_BUF_LEN];
  short sample;
  int total = numSamples * file->numChannels;

  for (i = 0; i < total; i++) {
    if (bytePos == WAVE_BUF_LEN) {
      writeBytes(file, bytes, bytePos);
      bytePos = 0;
    }
    sample = buffer[i];
    bytes[bytePos++] = sample;
    bytes[bytePos++] = sample >> 8;
  }
  if (bytePos != 0) {
    writeBytes(file, bytes, bytePos);
  }
  return file->failed;
}
