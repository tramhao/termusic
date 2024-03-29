////////////////////////////////////////////////////////////////////////////////
///
/// A class for parsing the 'soundstretch' application command line parameters
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

#ifndef RUNPARAMETERS_H
#define RUNPARAMETERS_H

#include <string>
#include "STTypes.h"
#include "SS_CharTypes.h"
#include "WavFile.h"

namespace soundstretch
{

/// Parses command line parameters into program parameters
class RunParameters
{
private:
    void throwIllegalParamExp(const STRING& str) const;
    void throwLicense() const;
    void parseSwitchParam(const STRING& str);
    void checkLimits();
    float parseSwitchValue(const STRING& tr) const;

public:
    STRING inFileName;
    STRING outFileName;
    float tempoDelta{ 0 };
    float pitchDelta{ 0 };
    float rateDelta{ 0 };
    int   quick{ 0 };
    int   noAntiAlias{ 0 };
    float goalBPM{ 0 };
    bool  detectBPM{ false };
    bool  speech{ false };

    RunParameters(int nParams, const CHARTYPE* paramStr[]);
};

}

#endif
