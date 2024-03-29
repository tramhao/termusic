#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

//TODO: Can bindgen generate these?
pub mod settings {
    /// Enable/disable anti-alias filter in pitch transposer (0 = disable)
    pub const SETTING_USE_AA_FILTER: i32 = 0;

    /// Pitch transposer anti-alias filter length (8 .. 128 taps, default = 32)
    pub const SETTING_AA_FILTER_LENGTH: i32 = 1;

    /// Enable/disable quick seeking algorithm in tempo changer routine
    /// (enabling quick seeking lowers CPU utilization but causes a minor sound
    ///  quality compromising)
    pub const SETTING_USE_QUICKSEEK: i32 = 2;

    /// Time-stretch algorithm single processing sequence length in milliseconds. This determines
    /// to how long sequences the original sound is chopped in the time-stretch algorithm.
    /// See "STTypes.h" or README for more information.
    pub const SETTING_SEQUENCE_MS: i32 = 3;

    /// Time-stretch algorithm seeking window length in milliseconds for algorithm that finds the
    /// best possible overlapping location. This determines from how wide window the algorithm
    /// may look for an optimal joining location when mixing the sound sequences back together.
    /// See "STTypes.h" or README for more information.
    pub const SETTING_SEEKWINDOW_MS: i32 = 4;

    /// Time-stretch algorithm overlap length in milliseconds. When the chopped sound sequences
    /// are mixed back together, to form a continuous sound stream, this parameter defines over
    /// how long period the two consecutive sequences are let to overlap each other.
    /// See "STTypes.h" or README for more information.
    pub const SETTING_OVERLAP_MS: i32 = 5;

    /// Call "getSetting" with this ID to query processing sequence size in samples.
    /// This value gives approximate value of how many input samples you'll need to
    /// feed into SoundTouch after initial buffering to get out a new batch of
    /// output samples.
    ///
    /// This value does not include initial buffering at beginning of a new processing
    /// stream, use SETTING_INITIAL_LATENCY to get the initial buffering size.
    ///
    /// Notices:
    /// - This is read-only parameter, i.e. setSetting ignores this parameter
    /// - This parameter value is not constant but change depending on
    ///   tempo/pitch/rate/samplerate settings.
    pub const SETTING_NOMINAL_INPUT_SEQUENCE: i32 = 6;

    /// Call "getSetting" with this ID to query nominal average processing output
    /// size in samples. This value tells approcimate value how many output samples
    /// SoundTouch outputs once it does DSP processing run for a batch of input samples.
    ///
    /// Notices:
    /// - This is read-only parameter, i.e. setSetting ignores this parameter
    /// - This parameter value is not constant but change depending on
    ///   tempo/pitch/rate/samplerate settings.
    pub const SETTING_NOMINAL_OUTPUT_SEQUENCE: i32 = 7;

    /// Call "getSetting" with this ID to query initial processing latency, i.e.
    /// approx. how many samples you'll need to enter to SoundTouch pipeline before
    /// you can expect to get first batch of ready output samples out.
    ///
    /// After the first output batch, you can then expect to get approx.
    /// SETTING_NOMINAL_OUTPUT_SEQUENCE ready samples out for every
    /// SETTING_NOMINAL_INPUT_SEQUENCE samples that you enter into SoundTouch.
    ///
    /// Example:
    ///     processing with parameter -tempo=5
    ///     => initial latency = 5509 samples
    ///        input sequence  = 4167 samples
    ///        output sequence = 3969 samples
    ///
    /// Accordingly, you can expect to feed in approx. 5509 samples at beginning of
    /// the stream, and then you'll get out the first 3969 samples. After that, for
    /// every approx. 4167 samples that you'll put in, you'll receive again approx.
    /// 3969 samples out.
    ///
    /// This also means that average latency during stream processing is
    /// INITIAL_LATENCY-OUTPUT_SEQUENCE/2, in the above example case 5509-3969/2
    /// = 3524 samples
    ///
    /// Notices:
    /// - This is read-only parameter, i.e. setSetting ignores this parameter
    /// - This parameter value is not constant but change depending on
    ///   tempo/pitch/rate/samplerate settings.
    pub const SETTING_INITIAL_LATENCY: i32 = 8;
}

#[cfg(test)]
mod tests {
    use crate::{root::soundtouch, settings};

    #[test]
    fn test_init() {
        unsafe {
            let mut soundtouch = soundtouch::SoundTouch::new();
            let original = soundtouch.getSetting(settings::SETTING_AA_FILTER_LENGTH);
            soundtouch.setSetting(settings::SETTING_AA_FILTER_LENGTH, 128);
            assert!(soundtouch.getSetting(settings::SETTING_AA_FILTER_LENGTH) == 128);
            soundtouch.setSetting(settings::SETTING_AA_FILTER_LENGTH, original);
            assert!(soundtouch.getSetting(settings::SETTING_AA_FILTER_LENGTH) == original);
            soundtouch.setPitchOctaves(1.0);
            let input_samples = soundtouch.getSetting(settings::SETTING_NOMINAL_INPUT_SEQUENCE);
            let output_samples = soundtouch.getSetting(settings::SETTING_NOMINAL_OUTPUT_SEQUENCE);
            assert!(input_samples == output_samples);
        }
    }
}
