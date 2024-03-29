////////////////////////////////////////////////////////////////////////////////
///
/// DllTest.cpp : This is small app main routine used for testing sound processing
/// with SoundTouch.dll API
///
/// Author        : Copyright (c) Olli Parviainen
/// Author e-mail : oparviai 'at' iki.fi
/// SoundTouch WWW: http://www.surina.net/soundtouch
///
////////////////////////////////////////////////////////////////////////////////

#include <string>
#include <iostream>
#include <fstream>
#include "../SoundTouchDLL.h"
#include "../../SoundStretch/WavFile.h"

using namespace std;
using namespace soundstretch;

// DllTest main
int wmain(int argc, const wchar_t *argv[])
{
    // Check program arguments
    if (argc < 4)
    {
        cout << "Too few arguments. Usage: DllTest [infile.wav] [outfile.wav] [sampletype]" << endl;
        return -1;
    }

    wstring inFileName = argv[1];
    wstring outFileName = argv[2];
    wstring str_sampleType = argv[3];

    bool floatSample;
    if (str_sampleType == L"float")
    {
        floatSample = true;
    }
    else if (str_sampleType == L"short")
    {
        floatSample = false;
    }
    else
    { 
        cerr << "Missing or invalid sampletype. Expected either short or float" << endl;
        return -1;
    }

    try
    {
        // Open input & output WAV files
        WavInFile inFile(inFileName);
        int numChannels = inFile.getNumChannels();
        int sampleRate = inFile.getSampleRate();
        WavOutFile outFile(outFileName, sampleRate, inFile.getNumBits(), numChannels);

        // Create SoundTouch DLL instance
        HANDLE st = soundtouch_createInstance();
        soundtouch_setChannels(st, numChannels);
        soundtouch_setSampleRate(st, sampleRate);
        soundtouch_setPitchSemiTones(st, 2);

        cout << "processing with soundtouch.dll routines";

        if (floatSample)
        {
            // Process file with SoundTouch.DLL float sample (default) API
            float fbuffer[2048];
            int nmax = 2048 / numChannels;

            cout << " using float api ..." << endl;
            while (inFile.eof() == false)
            {
                int n = inFile.read(fbuffer, nmax * numChannels) / numChannels;
                soundtouch_putSamples(st, fbuffer, n);
                do
                {
                    n = soundtouch_receiveSamples(st, fbuffer, nmax);
                    outFile.write(fbuffer, n * numChannels);
                } while (n > 0);
            }
        }
        else
        {
            // Process file with SoundTouch.DLL int16 (short) sample API.
            // Notice that SoundTouch.dll does internally processing using floating
            // point routines so the int16 API is not any faster, but provided for 
            // convenience.
            short i16buffer[2048];
            int nmax = 2048 / numChannels;

            cout << " using i16 api ..." << endl;
            while (inFile.eof() == false)
            {
                int n = inFile.read(i16buffer, nmax * numChannels) / numChannels;
                soundtouch_putSamples_i16(st, i16buffer, n);
                do
                {
                    n = soundtouch_receiveSamples_i16(st, i16buffer, nmax);
                    outFile.write(i16buffer, n * numChannels);
                } while (n > 0);
            }
        }

        soundtouch_destroyInstance(st);
        cout << "done." << endl;
    }
    catch (const runtime_error &e)
    {
        cerr << e.what() << endl;
    }

    return 0;
}
