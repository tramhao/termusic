@REM 
@REM SoundTouch & SoundStretch Build script for Win32 platform
@REM 
@REM You'll need Visual C++ 6.0 installed to compile - also execute the 
@REM "vcvars32.bat" in VC install directotry before running this one.
@REM 
@REM Copyright (c) Olli Parviainen
@REM

@if "%DevEnvDir%"=="" goto nodevdir

@rem devenv source\SoundStretch\SoundStretch.sln /upgrade
devenv source\SoundStretch\SoundStretch.sln /build "Debug|Win32"
devenv source\SoundStretch\SoundStretch.sln /build "Release|Win32"
devenv source\SoundStretch\SoundStretch.sln /build "Debug|x64"
devenv source\SoundStretch\SoundStretch.sln /build "Release|x64"

@rem devenv source\SoundTouchDll\SoundTouchDll.sln /upgrade
devenv source\SoundTouchDll\SoundTouchDll.sln /build "Debug|Win32"
devenv source\SoundTouchDll\SoundTouchDll.sln /build "Release|Win32"
devenv source\SoundTouchDll\SoundTouchDll.sln /build "Debug|x64"
devenv source\SoundTouchDll\SoundTouchDll.sln /build "Release|x64"

@goto end


:nodevdir

@echo off
echo ****************************************************************************
echo **
echo ** ERROR: Visual Studio path not set.
echo **
echo ** Open "tools"->"Developer Command Line" from Visual Studio IDE, or
echo ** run "vcvars32.bat" from Visual Studio installation dir, e.g.
echo ** "C:\Program Files (x86)\Microsoft Visual Studio xxx\VC\bin",
echo ** then try again.
echo **
echo ****************************************************************************

:end
