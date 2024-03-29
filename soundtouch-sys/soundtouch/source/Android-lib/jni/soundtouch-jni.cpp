////////////////////////////////////////////////////////////////////////////////
///
/// Example Interface class for SoundTouch native compilation
///
/// Author        : Copyright (c) Olli Parviainen
/// Author e-mail : oparviai 'at' iki.fi
/// WWW           : http://www.surina.net
///
////////////////////////////////////////////////////////////////////////////////

#include <jni.h>
#include <android/log.h>
#include <stdexcept>
#include <string>

using namespace std;

#include "../../../include/SoundTouch.h"
#include "../source/SoundStretch/WavFile.h"

#define LOGV(...)   __android_log_print((int)ANDROID_LOG_INFO, "SOUNDTOUCH", __VA_ARGS__)
//#define LOGV(...)


// String for keeping possible c++ exception error messages. Notice that this isn't
// thread-safe but it's expected that exceptions are special situations that won't
// occur in several threads in parallel.
static string _errMsg = "";


#define DLL_PUBLIC __attribute__ ((visibility ("default")))
#define BUFF_SIZE 4096


using namespace soundtouch;


// Set error message to return
static void _setErrmsg(const char *msg)
{
	_errMsg = msg;
}

#if 0   // apparently following workaround not needed any more with concurrent Android SDKs
#ifdef _OPENMP

#include <pthread.h>
extern pthread_key_t gomp_tls_key;
static void * _p_gomp_tls = nullptr;

/// Function to initialize threading for OpenMP.
///
/// This is a workaround for bug in Android NDK v10 regarding OpenMP: OpenMP works only if
/// called from the Android App main thread because in the main thread the gomp_tls storage is
/// properly set, however, Android does not properly initialize gomp_tls storage for other threads.
/// Thus if OpenMP routines are invoked from some other thread than the main thread,
/// the OpenMP routine will crash the application due to nullptr access on uninitialized storage.
///
/// This workaround stores the gomp_tls storage from main thread, and copies to other threads.
/// In order this to work, the Application main thread needws to call at least "getVersionString"
/// routine.
static int _init_threading(bool warn)
{
	void *ptr = pthread_getspecific(gomp_tls_key);
	LOGV("JNI thread-specific TLS storage %ld", (long)ptr);
	if (ptr == nullptr)
	{
		LOGV("JNI set missing TLS storage to %ld", (long)_p_gomp_tls);
		pthread_setspecific(gomp_tls_key, _p_gomp_tls);
	}
	else
	{
		LOGV("JNI store this TLS storage");
		_p_gomp_tls = ptr;
	}
	// Where critical, show warning if storage still not properly initialized
	if ((warn) && (_p_gomp_tls == nullptr))
	{
		_setErrmsg("Error - OpenMP threading not properly initialized: Call SoundTouch.getVersionString() from the App main thread!");
		return -1;
	}
	return 0;
}

#else
static int _init_threading(bool warn)
{
	// do nothing if not OpenMP build
	return 0;
}
#endif

#endif

// Processes the sound file
static void _processFile(SoundTouch *pSoundTouch, const char *inFileName, const char *outFileName)
{
    int nSamples;
    int nChannels;
    int buffSizeSamples;
    SAMPLETYPE sampleBuffer[BUFF_SIZE];

    // open input file
    WavInFile inFile(inFileName);
    int sampleRate = inFile.getSampleRate();
    int bits = inFile.getNumBits();
    nChannels = inFile.getNumChannels();

    // create output file
    WavOutFile outFile(outFileName, sampleRate, bits, nChannels);

    pSoundTouch->setSampleRate(sampleRate);
    pSoundTouch->setChannels(nChannels);

    assert(nChannels > 0);
    buffSizeSamples = BUFF_SIZE / nChannels;

    // Process samples read from the input file
    while (inFile.eof() == 0)
    {
        int num;

        // Read a chunk of samples from the input file
        num = inFile.read(sampleBuffer, BUFF_SIZE);
        nSamples = num / nChannels;

        // Feed the samples into SoundTouch processor
        pSoundTouch->putSamples(sampleBuffer, nSamples);

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
            nSamples = pSoundTouch->receiveSamples(sampleBuffer, buffSizeSamples);
            outFile.write(sampleBuffer, nSamples * nChannels);
        } while (nSamples != 0);
    }

    // Now the input file is processed, yet 'flush' few last samples that are
    // hiding in the SoundTouch's internal processing pipeline.
    pSoundTouch->flush();
    do
    {
        nSamples = pSoundTouch->receiveSamples(sampleBuffer, buffSizeSamples);
        outFile.write(sampleBuffer, nSamples * nChannels);
    } while (nSamples != 0);
}



extern "C" DLL_PUBLIC jstring Java_net_surina_soundtouch_SoundTouch_getVersionString(JNIEnv *env, jobject thiz)
{
    const char *verStr;

    LOGV("JNI call SoundTouch.getVersionString");

    // Call example SoundTouch routine
    verStr = SoundTouch::getVersionString();

    // gomp_tls storage bug workaround - see comments in _init_threading() function!
    // update: apparently this is not needed any more with concurrent Android SDKs
    // _init_threading(false);

    int threads = 0;
	#pragma omp parallel
    {
		#pragma omp atomic
    	threads ++;
    }
    LOGV("JNI thread count %d", threads);

    // return version as string
    return env->NewStringUTF(verStr);
}



extern "C" DLL_PUBLIC jlong Java_net_surina_soundtouch_SoundTouch_newInstance(JNIEnv *env, jobject thiz)
{
	return (jlong)(new SoundTouch());
}


extern "C" DLL_PUBLIC void Java_net_surina_soundtouch_SoundTouch_deleteInstance(JNIEnv *env, jobject thiz, jlong handle)
{
	SoundTouch *ptr = (SoundTouch*)handle;
	delete ptr;
}


extern "C" DLL_PUBLIC void Java_net_surina_soundtouch_SoundTouch_setTempo(JNIEnv *env, jobject thiz, jlong handle, jfloat tempo)
{
	SoundTouch *ptr = (SoundTouch*)handle;
	ptr->setTempo(tempo);
}


extern "C" DLL_PUBLIC void Java_net_surina_soundtouch_SoundTouch_setPitchSemiTones(JNIEnv *env, jobject thiz, jlong handle, jfloat pitch)
{
	SoundTouch *ptr = (SoundTouch*)handle;
	ptr->setPitchSemiTones(pitch);
}


extern "C" DLL_PUBLIC void Java_net_surina_soundtouch_SoundTouch_setSpeed(JNIEnv *env, jobject thiz, jlong handle, jfloat speed)
{
	SoundTouch *ptr = (SoundTouch*)handle;
	ptr->setRate(speed);
}


extern "C" DLL_PUBLIC jstring Java_net_surina_soundtouch_SoundTouch_getErrorString(JNIEnv *env, jobject thiz)
{
	jstring result = env->NewStringUTF(_errMsg.c_str());
	_errMsg.clear();

	return result;
}


extern "C" DLL_PUBLIC int Java_net_surina_soundtouch_SoundTouch_processFile(JNIEnv *env, jobject thiz, jlong handle, jstring jinputFile, jstring joutputFile)
{
	SoundTouch *ptr = (SoundTouch*)handle;

	const char *inputFile = env->GetStringUTFChars(jinputFile, 0);
	const char *outputFile = env->GetStringUTFChars(joutputFile, 0);

	LOGV("JNI process file %s", inputFile);

    /// gomp_tls storage bug workaround - see comments in _init_threading() function!
    // update: apparently this is not needed any more with concurrent Android SDKs
    // if (_init_threading(true)) return -1;

	try
	{
		_processFile(ptr, inputFile, outputFile);
	}
	catch (const runtime_error &e)
    {
		const char *err = e.what();
        // An exception occurred during processing, return the error message
    	LOGV("JNI exception in SoundTouch::processFile: %s", err);
        _setErrmsg(err);
        return -1;
    }


	env->ReleaseStringUTFChars(jinputFile, inputFile);
	env->ReleaseStringUTFChars(joutputFile, outputFile);

	return 0;
}
