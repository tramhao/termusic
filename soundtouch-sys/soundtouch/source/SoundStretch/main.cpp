////////////////////////////////////////////////////////////////////////////////
///
/// SoundStretch main routine.
///
/// Author        : Copyright (c) Olli Parviainen
/// Author e-mail : oparviai 'at' iki.fi
/// SoundTouch WWW: http://www.surina.net/soundtouch
///
////////////////////////////////////////////////////////////////////////////////
//
// License :
//
//  SoundTouch audio processing library
//  Copyright (c) Olli Parviainen
//
//  This library is free software; you can redistribute it and/or
//  modify it under the terms of the GNU Lesser General Public
//  License as published by the Free Software Foundation; either
//  version 2.1 of the License, or (at your option) any later version.
//
//  This library is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  Lesser General Public License for more details.
//
//  You should have received a copy of the GNU Lesser General Public
//  License along with this library; if not, write to the Free Software
//  Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA
//
////////////////////////////////////////////////////////////////////////////////

#include <iostream>
#include <memory>
#include <stdexcept>
#include <string>
#include <cstdio>
#include <ctime>
#include "RunParameters.h"
#include "WavFile.h"
#include "SoundTouch.h"
#include "BPMDetect.h"

using namespace soundtouch;
using namespace std;

namespace soundstretch
{

// Processing chunk size (size chosen to be divisible by 2, 4, 6, 8, 10, 12, 14, 16 channels ...)
#define BUFF_SIZE           6720

#if _WIN32
#include <io.h>
#include <fcntl.h>

// Macro for Win32 standard input/output stream support: Sets a file stream into binary mode
#define SET_STREAM_TO_BIN_MODE(f) (_setmode(_fileno(f), _O_BINARY))
#else
    // Not needed for GNU environment... 
#define SET_STREAM_TO_BIN_MODE(f) {}
#endif


static const char _helloText[] =
    "\n"
    "   SoundStretch v%s -  Copyright (c) Olli Parviainen\n"
    "=========================================================\n"
    "author e-mail: <oparviai"
    "@"
    "iki.fi> - WWW: http://www.surina.net/soundtouch\n"
    "\n"
    "This program is subject to (L)GPL license. Run \"soundstretch -license\" for\n"
    "more information.\n"
    "\n";

static void openFiles(unique_ptr<WavInFile>& inFile, unique_ptr<WavOutFile>& outFile, const RunParameters& params)
{
    if (params.inFileName == STRING_CONST("stdin"))
    {
        // used 'stdin' as input file
        SET_STREAM_TO_BIN_MODE(stdin);
        inFile = make_unique<WavInFile>(stdin);
    }
    else
    {
        // open input file...
        inFile = make_unique<WavInFile>(params.inFileName.c_str());
    }

    // ... open output file with same sound parameters
    const int bits = (int)inFile->getNumBits();
    const int samplerate = (int)inFile->getSampleRate();
    const int channels = (int)inFile->getNumChannels();

    if (!params.outFileName.empty())
    {
        if (params.outFileName == STRING_CONST("stdout"))
        {
            SET_STREAM_TO_BIN_MODE(stdout);
            outFile = make_unique<WavOutFile>(stdout, samplerate, bits, channels);
        }
        else
        {
            outFile = make_unique<WavOutFile>(params.outFileName.c_str(), samplerate, bits, channels);
        }
    }
}


// Sets the 'SoundTouch' object up according to input file sound format & 
// command line parameters
static void setup(SoundTouch& soundTouch, const WavInFile& inFile, const RunParameters& params)
{
    const int sampleRate = (int)inFile.getSampleRate();
    const int channels = (int)inFile.getNumChannels();
    soundTouch.setSampleRate(sampleRate);
    soundTouch.setChannels(channels);

    soundTouch.setTempoChange(params.tempoDelta);
    soundTouch.setPitchSemiTones(params.pitchDelta);
    soundTouch.setRateChange(params.rateDelta);

    soundTouch.setSetting(SETTING_USE_QUICKSEEK, params.quick);
    soundTouch.setSetting(SETTING_USE_AA_FILTER, !(params.noAntiAlias));

    if (params.speech)
    {
        // use settings for speech processing
        soundTouch.setSetting(SETTING_SEQUENCE_MS, 40);
        soundTouch.setSetting(SETTING_SEEKWINDOW_MS, 15);
        soundTouch.setSetting(SETTING_OVERLAP_MS, 8);
        fprintf(stderr, "Tune processing parameters for speech processing.\n");
    }

    // print processing information
    if (!params.outFileName.empty())
    {
#ifdef SOUNDTOUCH_INTEGER_SAMPLES
        fprintf(stderr, "Uses 16bit integer sample type in processing.\n\n");
#else
#ifndef SOUNDTOUCH_FLOAT_SAMPLES
#error "Sampletype not defined"
#endif
        fprintf(stderr, "Uses 32bit floating point sample type in processing.\n\n");
#endif
        // print processing information only if outFileName given i.e. some processing will happen
        fprintf(stderr, "Processing the file with the following changes:\n");
        fprintf(stderr, "  tempo change = %+g %%\n", params.tempoDelta);
        fprintf(stderr, "  pitch change = %+g semitones\n", params.pitchDelta);
        fprintf(stderr, "  rate change  = %+g %%\n\n", params.rateDelta);
        fprintf(stderr, "Working...");
    }
    else
    {
        // outFileName not given
        fprintf(stderr, "Warning: output file name missing, won't output anything.\n\n");
    }

    fflush(stderr);
}


// Processes the sound
static void process(SoundTouch& soundTouch, WavInFile& inFile, WavOutFile& outFile)
{
    SAMPLETYPE sampleBuffer[BUFF_SIZE];
    int nSamples;

    const int nChannels = (int)inFile.getNumChannels();
    assert(nChannels > 0);
    const int buffSizeSamples = BUFF_SIZE / nChannels;

    // Process samples read from the input file
    while (inFile.eof() == 0)
    {
        // Read a chunk of samples from the input file
        const int num = inFile.read(sampleBuffer, BUFF_SIZE);
        int nSamples = num / (int)inFile.getNumChannels();

        // Feed the samples into SoundTouch processor
        soundTouch.putSamples(sampleBuffer, nSamples);

        // Read ready samples from SoundTouch processor & write them output file.
        // NOTES:
        // - 'receiveSamples' doesn't necessarily return any samples at all
        //   during some rounds!
        // - On the other hand, during some round 'receiveSamples' may have more
        //   ready samples than would fit into 'sampleBuffer', and for this reason 
        //   the 'receiveSamples' call is iterated for as many times as it
        //   outputs samples.
        do
        {
            nSamples = soundTouch.receiveSamples(sampleBuffer, buffSizeSamples);
            outFile.write(sampleBuffer, nSamples * nChannels);
        } while (nSamples != 0);
    }

    // Now the input file is processed, yet 'flush' few last samples that are
    // hiding in the SoundTouch's internal processing pipeline.
    soundTouch.flush();
    do
    {
        nSamples = soundTouch.receiveSamples(sampleBuffer, buffSizeSamples);
        outFile.write(sampleBuffer, nSamples * nChannels);
    } while (nSamples != 0);
}


// Detect BPM rate of inFile and adjust tempo setting accordingly if necessary
static void detectBPM(WavInFile& inFile, RunParameters& params)
{
    BPMDetect bpm(inFile.getNumChannels(), inFile.getSampleRate());
    SAMPLETYPE sampleBuffer[BUFF_SIZE];

    // detect bpm rate
    fprintf(stderr, "Detecting BPM rate...");
    fflush(stderr);

    const int nChannels = (int)inFile.getNumChannels();
    int readSize = BUFF_SIZE - BUFF_SIZE % nChannels;   // round read size down to multiple of num.channels 

    // Process the 'inFile' in small blocks, repeat until whole file has 
    // been processed
    while (inFile.eof() == 0)
    {
        // Read sample data from input file
        const int num = inFile.read(sampleBuffer, readSize);

        // Enter the new samples to the bpm analyzer class
        const int samples = num / nChannels;
        bpm.inputSamples(sampleBuffer, samples);
    }

    // Now the whole song data has been analyzed. Read the resulting bpm.
    const float bpmValue = bpm.getBpm();
    fprintf(stderr, "Done!\n");

    // rewind the file after bpm detection
    inFile.rewind();

    if (bpmValue > 0)
    {
        fprintf(stderr, "Detected BPM rate %.1f\n\n", bpmValue);
    }
    else
    {
        fprintf(stderr, "Couldn't detect BPM rate.\n\n");
        return;
    }

    if (params.goalBPM > 0)
    {
        // adjust tempo to given bpm
        params.tempoDelta = (params.goalBPM / bpmValue - 1.0f) * 100.0f;
        fprintf(stderr, "The file will be converted to %.1f BPM\n\n", params.goalBPM);
    }
}

void ss_main(RunParameters& params)
{
    unique_ptr<WavInFile> inFile;
    unique_ptr<WavOutFile> outFile;
    SoundTouch soundTouch;

    fprintf(stderr, _helloText, soundTouch.getVersionString());

    // Open input & output files
    openFiles(inFile, outFile, params);

    if (params.detectBPM == true)
    {
        // detect sound BPM (and adjust processing parameters
        //  accordingly if necessary)
        detectBPM(*inFile, params);
    }

    // Setup the 'SoundTouch' object for processing the sound
    setup(soundTouch, *inFile, params);

    // clock_t cs = clock();    // for benchmarking processing duration
    // Process the sound
    if (inFile && outFile)
    {
        process(soundTouch, *inFile, *outFile);
    }
    // clock_t ce = clock();    // for benchmarking processing duration
    // printf("duration: %lf\n", (double)(ce-cs)/CLOCKS_PER_SEC);

    fprintf(stderr, "Done!\n");
}

}

#if _WIN32
int wmain(int argc, const wchar_t* args[])
#else
int main(int argc, const char* args[])
#endif
{
    try
    {
        soundstretch::RunParameters params(argc, args);
        soundstretch::ss_main(params);
    }
    catch (const runtime_error& e)
    {
        fprintf(stderr, "%s\n", e.what());
        return -1;
    }
    return 0;
}
