////////////////////////////////////////////////////////////////////////////////
///
/// Example class that invokes native SoundTouch routines through the JNI
/// interface.
///
/// Author        : Copyright (c) Olli Parviainen
/// Author e-mail : oparviai 'at' iki.fi
/// WWW           : http://www.surina.net
///
////////////////////////////////////////////////////////////////////////////////

package net.surina.soundtouch;

public final class SoundTouch
{
    // Native interface function that returns SoundTouch version string.
    // This invokes the native c++ routine defined in "soundtouch-jni.cpp".
    public native final static String getVersionString();
    
    private native final void setTempo(long handle, float tempo);

    private native final void setPitchSemiTones(long handle, float pitch);
    
    private native final void setSpeed(long handle, float speed);

    private native final int processFile(long handle, String inputFile, String outputFile);

    public native final static String getErrorString();

    private native final static long newInstance();
    
    private native final void deleteInstance(long handle);
    
    long handle = 0;
    
    
    public SoundTouch()
    {
    	handle = newInstance();    	
    }
    
    
    public void close()
    {
    	deleteInstance(handle);
    	handle = 0;
    }


    public void setTempo(float tempo)
    {
    	setTempo(handle, tempo);
    }


    public void setPitchSemiTones(float pitch)
    {
    	setPitchSemiTones(handle, pitch);
    }

    
    public void setSpeed(float speed)
    {
    	setSpeed(handle, speed);
    }


    public int processFile(String inputFile, String outputFile)
    {
    	return processFile(handle, inputFile, outputFile);
    }

    
    // Load the native library upon startup
    static
    {
        System.loadLibrary("soundtouch");
    }
}
