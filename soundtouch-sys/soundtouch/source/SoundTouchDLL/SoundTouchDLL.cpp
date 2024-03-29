//////////////////////////////////////////////////////////////////////////////
///
/// SoundTouch DLL wrapper - wraps SoundTouch routines into a Dynamic Load
/// Library interface.
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


#if defined(_WIN32) || defined(WIN32)
    #include <windows.h>

    // DLL main in Windows compilation
    BOOL APIENTRY DllMain( HANDLE hModule,
                           DWORD  ul_reason_for_call,
                           LPVOID lpReserved
                         )
    {
        switch (ul_reason_for_call)
        {
        case DLL_PROCESS_ATTACH:
        case DLL_THREAD_ATTACH:
        case DLL_THREAD_DETACH:
        case DLL_PROCESS_DETACH:
            break;
        }
        return TRUE;
    }
#endif

#include <limits.h>
#include <string.h>
#include "SoundTouchDLL.h"
#include "SoundTouch.h"
#include "BPMDetect.h"

using namespace soundtouch;

#ifdef SOUNDTOUCH_INTEGER_SAMPLES
    #error "error - compile the dll version with float samples"
#endif // SOUNDTOUCH_INTEGER_SAMPLES

//////////////

typedef struct
{
    DWORD dwMagic;
    SoundTouch *pst;
} STHANDLE;

typedef struct
{
    DWORD dwMagic;
    BPMDetect *pbpm;
    uint numChannels;
} BPMHANDLE;

#define STMAGIC  0x1770C001
#define BPMMAGIC 0x1771C10a

SOUNDTOUCHDLL_API HANDLE __cdecl soundtouch_createInstance()
{
    STHANDLE *tmp = new STHANDLE;

    if (tmp)
    {
        tmp->dwMagic = STMAGIC;
        tmp->pst = new SoundTouch();
        if (tmp->pst == nullptr)
        {
            delete tmp;
            tmp = nullptr;
        }
    }
    return (HANDLE)tmp;
}


SOUNDTOUCHDLL_API void __cdecl soundtouch_destroyInstance(HANDLE h)
{
    STHANDLE *sth = (STHANDLE*)h;
    if (sth->dwMagic != STMAGIC) return;

    sth->dwMagic = 0;
    if (sth->pst) delete sth->pst;
    sth->pst = nullptr;
    delete sth;
}


/// Get SoundTouch library version string
SOUNDTOUCHDLL_API const char *__cdecl soundtouch_getVersionString()
{
    return SoundTouch::getVersionString();
}


/// Get SoundTouch library version string - alternative function for
/// environments that can't properly handle character string as return value
SOUNDTOUCHDLL_API void __cdecl soundtouch_getVersionString2(char* versionString, int bufferSize)
{
    strncpy(versionString, SoundTouch::getVersionString(), bufferSize - 1);
    versionString[bufferSize - 1] = 0;
}


/// Get SoundTouch library version Id
SOUNDTOUCHDLL_API uint __cdecl soundtouch_getVersionId()
{
    return SoundTouch::getVersionId();
}

/// Sets new rate control value. Normal rate = 1.0, smaller values
/// represent slower rate, larger faster rates.
SOUNDTOUCHDLL_API void __cdecl soundtouch_setRate(HANDLE h, float newRate)
{
    STHANDLE *sth = (STHANDLE*)h;
    if (sth->dwMagic != STMAGIC) return;

    sth->pst->setRate(newRate);
}


/// Sets new tempo control value. Normal tempo = 1.0, smaller values
/// represent slower tempo, larger faster tempo.
SOUNDTOUCHDLL_API void __cdecl soundtouch_setTempo(HANDLE h, float newTempo)
{
    STHANDLE *sth = (STHANDLE*)h;
    if (sth->dwMagic != STMAGIC) return;

    sth->pst->setTempo(newTempo);
}

/// Sets new rate control value as a difference in percents compared
/// to the original rate (-50 .. +100 %)
SOUNDTOUCHDLL_API void __cdecl soundtouch_setRateChange(HANDLE h, float newRate)
{
    STHANDLE *sth = (STHANDLE*)h;
    if (sth->dwMagic != STMAGIC) return;

    sth->pst->setRateChange(newRate);
}

/// Sets new tempo control value as a difference in percents compared
/// to the original tempo (-50 .. +100 %)
SOUNDTOUCHDLL_API void __cdecl soundtouch_setTempoChange(HANDLE h, float newTempo)
{
    STHANDLE *sth = (STHANDLE*)h;
    if (sth->dwMagic != STMAGIC) return;

    sth->pst->setTempoChange(newTempo);
}

/// Sets new pitch control value. Original pitch = 1.0, smaller values
/// represent lower pitches, larger values higher pitch.
SOUNDTOUCHDLL_API void __cdecl soundtouch_setPitch(HANDLE h, float newPitch)
{
    STHANDLE *sth = (STHANDLE*)h;
    if (sth->dwMagic != STMAGIC) return;

    sth->pst->setPitch(newPitch);
}

/// Sets pitch change in octaves compared to the original pitch
/// (-1.00 .. +1.00)
SOUNDTOUCHDLL_API void __cdecl soundtouch_setPitchOctaves(HANDLE h, float newPitch)
{
    STHANDLE *sth = (STHANDLE*)h;
    if (sth->dwMagic != STMAGIC) return;

    sth->pst->setPitchOctaves(newPitch);
}

/// Sets pitch change in semi-tones compared to the original pitch
/// (-12 .. +12)
SOUNDTOUCHDLL_API void __cdecl soundtouch_setPitchSemiTones(HANDLE h, float newPitch)
{
    STHANDLE *sth = (STHANDLE*)h;
    if (sth->dwMagic != STMAGIC) return;

    sth->pst->setPitchSemiTones(newPitch);
}


/// Sets the number of channels, 1 = mono, 2 = stereo
SOUNDTOUCHDLL_API int __cdecl soundtouch_setChannels(HANDLE h, uint numChannels)
{
    STHANDLE *sth = (STHANDLE*)h;
    if (sth->dwMagic != STMAGIC) return 0;

    try
    {
        sth->pst->setChannels(numChannels);
    }
    catch (const std::exception&)
    {
        return 0;
    }
    return 1;
}

/// Sets sample rate.
SOUNDTOUCHDLL_API int __cdecl soundtouch_setSampleRate(HANDLE h, uint srate)
{
    STHANDLE *sth = (STHANDLE*)h;
    if (sth->dwMagic != STMAGIC) return 0;

    try
    {
        sth->pst->setSampleRate(srate);
    }
    catch (const std::exception&)
    {
        return 0;
    }
    return 1;
}

/// Flushes the last samples from the processing pipeline to the output.
/// Clears also the internal processing buffers.
//
/// Note: This function is meant for extracting the last samples of a sound
/// stream. This function may introduce additional blank samples in the end
/// of the sound stream, and thus it's not recommended to call this function
/// in the middle of a sound stream.
SOUNDTOUCHDLL_API int __cdecl soundtouch_flush(HANDLE h)
{
    STHANDLE *sth = (STHANDLE*)h;
    if (sth->dwMagic != STMAGIC) return 0;

    try
    {
        sth->pst->flush();
    }
    catch (const std::exception&)
    {
        return 0;
    }
    return 1;
}

/// Adds 'numSamples' pcs of samples from the 'samples' memory position into
/// the input of the object. Notice that sample rate _has_to_ be set before
/// calling this function, otherwise throws a runtime_error exception.
SOUNDTOUCHDLL_API int __cdecl soundtouch_putSamples(HANDLE h,
        const SAMPLETYPE *samples,      ///< Pointer to sample buffer.
        unsigned int numSamples         ///< Number of samples in buffer. Notice
                                        ///< that in case of stereo-sound a single sample
                                        ///< contains data for both channels.
        )
{
    STHANDLE *sth = (STHANDLE*)h;
    if (sth->dwMagic != STMAGIC) return 0;

    try
    {
        sth->pst->putSamples(samples, numSamples);
    }
    catch (const std::exception&)
    {
        return 0;
    }
    return 1;
}

/// int16 version of soundtouch_putSamples(): This accept int16 (short) sample data
/// and internally converts it to float format before processing
SOUNDTOUCHDLL_API void __cdecl soundtouch_putSamples_i16(HANDLE h,
        const short *samples,       ///< Pointer to sample buffer.
        unsigned int numSamples     ///< Number of sample frames in buffer. Notice
                                    ///< that in case of multi-channel sound a single sample
                                    ///< contains data for all channels.
        )
{
    STHANDLE *sth = (STHANDLE*)h;
    if (sth->dwMagic != STMAGIC) return;

    uint numChannels = sth->pst->numChannels();

    // iterate until all samples converted & put to SoundTouch object
    while (numSamples > 0)
    {
        float convert[8192];    // allocate temporary conversion buffer from stack

        // how many multichannel samples fit into 'convert' buffer:
        uint convSamples = 8192 / numChannels;

        // convert max 'nround' values at a time to guarantee that these fit in the 'convert' buffer
        uint n = (numSamples > convSamples) ? convSamples : numSamples;
        for (uint i = 0; i < n * numChannels; i++)
        {
            convert[i] = samples[i];
        }
        // put the converted samples into SoundTouch
        sth->pst->putSamples(convert, n);

        numSamples -= n;
        samples += n * numChannels;
    }
}

/// Clears all the samples in the object's output and internal processing
/// buffers.
SOUNDTOUCHDLL_API void __cdecl soundtouch_clear(HANDLE h)
{
    STHANDLE *sth = (STHANDLE*)h;
    if (sth->dwMagic != STMAGIC) return;

    sth->pst->clear();
}

/// Changes a setting controlling the processing system behaviour. See the
/// 'SETTING_...' defines for available setting ID's.
///
/// \return 'nonzero' if the setting was successfully changed
SOUNDTOUCHDLL_API int __cdecl soundtouch_setSetting(HANDLE h,
        int settingId,   ///< Setting ID number. see SETTING_... defines.
        int value        ///< New setting value.
        )
{
    STHANDLE *sth = (STHANDLE*)h;
    if (sth->dwMagic != STMAGIC) return FALSE;

    return sth->pst->setSetting(settingId, value);
}

/// Reads a setting controlling the processing system behaviour. See the
/// 'SETTING_...' defines for available setting ID's.
///
/// \return the setting value.
SOUNDTOUCHDLL_API int __cdecl soundtouch_getSetting(HANDLE h,
        int settingId    ///< Setting ID number, see SETTING_... defines.
        )
{
    STHANDLE *sth = (STHANDLE*)h;
    if (sth->dwMagic != STMAGIC) return -1;

    return sth->pst->getSetting(settingId);
}


/// Returns number of samples currently unprocessed.
SOUNDTOUCHDLL_API uint __cdecl soundtouch_numUnprocessedSamples(HANDLE h)
{
    STHANDLE *sth = (STHANDLE*)h;
    if (sth->dwMagic != STMAGIC) return 0;

    return sth->pst->numUnprocessedSamples();
}


/// Receive ready samples from the processing pipeline.
///
/// if called with outBuffer=nullptr, just reduces amount of ready samples within the pipeline.
SOUNDTOUCHDLL_API uint __cdecl soundtouch_receiveSamples(HANDLE h,
        SAMPLETYPE *outBuffer,      ///< Buffer where to copy output samples.
        unsigned int maxSamples     ///< How many samples to receive at max.
        )
{
    STHANDLE *sth = (STHANDLE*)h;
    if (sth->dwMagic != STMAGIC) return 0;

    if (outBuffer)
    {
        return sth->pst->receiveSamples(outBuffer, maxSamples);
    }
    else
    {
        return sth->pst->receiveSamples(maxSamples);
    }
}


/// int16 version of soundtouch_receiveSamples(): This converts internal float samples
/// into int16 (short) return data type
SOUNDTOUCHDLL_API uint __cdecl soundtouch_receiveSamples_i16(HANDLE h,
        short *outBuffer,           ///< Buffer where to copy output samples.
        unsigned int maxSamples     ///< How many samples to receive at max.
        )
{
    STHANDLE *sth = (STHANDLE*)h;
    if (sth->dwMagic != STMAGIC) return 0;
    uint outTotal = 0;

    if (outBuffer == nullptr)
    {
        // only reduce sample count, not receive samples
        return sth->pst->receiveSamples(maxSamples);
    }

    uint numChannels = sth->pst->numChannels();

    // iterate until all samples converted & put to SoundTouch object
    while (maxSamples > 0)
    {
        float convert[8192];    // allocate temporary conversion buffer from stack

        // how many multichannel samples fit into 'convert' buffer:
        uint convSamples = 8192 / numChannels;

        // request max 'nround' values at a time to guarantee that these fit in the 'convert' buffer
        uint n = (maxSamples > convSamples) ? convSamples : maxSamples;

        uint out = sth->pst->receiveSamples(convert, n);

        // convert & saturate received samples to int16
        for (uint i = 0; i < out * numChannels; i++)
        {
            // first convert value to int32, then saturate to int16 min/max limits
            int value = (int)convert[i];
            value = (value < SHRT_MIN) ? SHRT_MIN : (value > SHRT_MAX) ? SHRT_MAX : value;
            outBuffer[i] = (short)value;
        }
        outTotal += out;
        if (out < n) break;  // didn't get as many as asked => no more samples available => break here

        maxSamples -= n;
        outBuffer += out * numChannels;
    }

    // return number of processed samples
    return outTotal;
}


/// Returns number of samples currently available.
SOUNDTOUCHDLL_API uint __cdecl soundtouch_numSamples(HANDLE h)
{
    STHANDLE *sth = (STHANDLE*)h;
    if (sth->dwMagic != STMAGIC) return 0;

    return sth->pst->numSamples();
}


/// Returns nonzero if there aren't any samples available for outputting.
SOUNDTOUCHDLL_API int __cdecl soundtouch_isEmpty(HANDLE h)
{
    STHANDLE *sth = (STHANDLE*)h;
    if (sth->dwMagic != STMAGIC) return -1;

    return sth->pst->isEmpty();
}


SOUNDTOUCHDLL_API HANDLE __cdecl bpm_createInstance(int numChannels, int sampleRate)
{
    BPMHANDLE *tmp = new BPMHANDLE;

    if (tmp)
    {
        tmp->dwMagic = BPMMAGIC;
        try
        {
            tmp->pbpm = new BPMDetect(numChannels, sampleRate);
        }
        catch (const std::exception&)
        {
            tmp->pbpm = nullptr;
        }
        if (tmp->pbpm == nullptr)
        {
            delete tmp;
            tmp = nullptr;
        }
    }
    return (HANDLE)tmp;
}


SOUNDTOUCHDLL_API void __cdecl bpm_destroyInstance(HANDLE h)
{
    BPMHANDLE *sth = (BPMHANDLE*)h;
    if (sth->dwMagic != BPMMAGIC) return;

    sth->dwMagic = 0;
    if (sth->pbpm) delete sth->pbpm;
    sth->pbpm = nullptr;
    delete sth;
}


/// Feed 'numSamples' sample frames from 'samples' into the BPM detection handler
SOUNDTOUCHDLL_API void __cdecl bpm_putSamples(HANDLE h,
        const float *samples,
        unsigned int numSamples)
{
    BPMHANDLE *bpmh = (BPMHANDLE*)h;
    if (bpmh->dwMagic != BPMMAGIC) return;

    bpmh->pbpm->inputSamples(samples, numSamples);
}


/// Feed 'numSamples' sample frames from 'samples' into the BPM detection handler.
/// 16bit int sample format version.
SOUNDTOUCHDLL_API void __cdecl bpm_putSamples_i16(HANDLE h,
        const short *samples,
        unsigned int numSamples)
{
    BPMHANDLE *bpmh = (BPMHANDLE*)h;
    if (bpmh->dwMagic != BPMMAGIC) return;

    uint numChannels = bpmh->numChannels;

    // iterate until all samples converted & put to SoundTouch object
    while (numSamples > 0)
    {
        float convert[8192];    // allocate temporary conversion buffer from stack

        // how many multichannel samples fit into 'convert' buffer:
        uint convSamples = 8192 / numChannels;

        // convert max 'nround' values at a time to guarantee that these fit in the 'convert' buffer
        uint n = (numSamples > convSamples) ? convSamples : numSamples;
        for (uint i = 0; i < n * numChannels; i++)
        {
            convert[i] = samples[i];
        }
        // put the converted samples into SoundTouch
        bpmh->pbpm->inputSamples(convert, n);

        numSamples -= n;
        samples += n * numChannels;
    }
}


/// Analyzes the results and returns the BPM rate. Use this function to read result
/// after whole song data has been input to the class by consecutive calls of
/// 'inputSamples' function.
///
/// \return Beats-per-minute rate, or zero if detection failed.
SOUNDTOUCHDLL_API float __cdecl bpm_getBpm(HANDLE h)
{
    BPMHANDLE *bpmh = (BPMHANDLE*)h;
    if (bpmh->dwMagic != BPMMAGIC) return 0;

    return bpmh->pbpm->getBpm();
}


/// Get beat position arrays. Note: The array includes also really low beat detection values 
/// in absence of clear strong beats. Consumer may wish to filter low values away.
/// - "pos" receive array of beat positions
/// - "values" receive array of beat detection strengths
/// - max_num indicates max.size of "pos" and "values" array.  
///
/// You can query a suitable array sized by calling this with nullptr in "pos" & "values".
///
/// \return number of beats in the arrays.
SOUNDTOUCHDLL_API int __cdecl bpm_getBeats(HANDLE h, float* pos, float* strength, int count)
{
	BPMHANDLE *bpmh = (BPMHANDLE *)h;
	if (bpmh->dwMagic != BPMMAGIC) return 0;

	return bpmh->pbpm->getBeats(pos, strength, count);
}
