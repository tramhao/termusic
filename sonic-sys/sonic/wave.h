/* Sonic library
   Copyright 2010
   Bill Cox
   This file is part of the Sonic Library.

   This file is licensed under the Apache 2.0 license. */

/* Support for reading and writing wave files. */

typedef struct waveFileStruct* waveFile;

waveFile openInputWaveFile(const char* fileName, int* sampleRate, int* numChannels);
waveFile openOutputWaveFile(const char* fileName, int sampleRate, int numChannels);
int closeWaveFile(waveFile file);
int readFromWaveFile(waveFile file, short* buffer, int maxSamples);
int writeToWaveFile(waveFile file, short* buffer, int numSamples);
