use std::ffi::CStr;

use soundtouch_sys::root::soundtouch;

pub use soundtouch_sys::settings;

pub struct SoundTouch(soundtouch_sys::root::soundtouch::SoundTouch, u32, u16);

unsafe impl Send for SoundTouch {}

impl SoundTouch {
    pub fn new(channels: u16, sample_rate: u32) -> Self {
        unsafe {
            let mut ret = SoundTouch(
                soundtouch_sys::root::soundtouch::SoundTouch::new(),
                sample_rate,
                channels,
            );
            ret.0.setSampleRate(sample_rate);
            ret.0.setChannels(channels as _);
            ret
        }
    }

    pub fn put_samples(&mut self, samples: &[f32]) {
        unsafe {
            soundtouch::SoundTouch_putSamples(
                &mut self.0 as *mut _ as _,
                samples.as_ptr(),
                samples.len() as u32 / self.2 as u32,
            );
        }
    }

    pub fn read_samples(&mut self, buffer: &mut [f32]) -> u32 {
        unsafe {
            soundtouch::SoundTouch_receiveSamples(
                &mut self.0 as *mut _ as _,
                buffer.as_mut_ptr(),
                buffer.len() as u32 / self.2 as u32,
            )
        }
    }

    pub fn get_version_string() -> String {
        unsafe {
            CStr::from_ptr(soundtouch::SoundTouch_getVersionString())
                .to_string_lossy()
                .to_string()
        }
    }
    pub fn get_version_id() -> u32 {
        unsafe { soundtouch::SoundTouch_getVersionId() }
    }
    pub fn set_rate(&mut self, new_rate: f64) {
        unsafe { self.0.setRate(new_rate) }
    }
    pub fn set_tempo(&mut self, new_tempo: f64) {
        unsafe { self.0.setTempo(new_tempo) }
    }
    pub fn set_rate_change(&mut self, new_rate: f64) {
        unsafe { self.0.setRateChange(new_rate) }
    }
    pub fn set_tempo_change(&mut self, new_tempo: f64) {
        unsafe { self.0.setTempoChange(new_tempo) }
    }
    pub fn set_pitch(&mut self, new_pitch: f64) {
        unsafe { self.0.setPitch(new_pitch) }
    }
    pub fn set_pitch_octaves(&mut self, new_pitch: f64) {
        unsafe { self.0.setPitchOctaves(new_pitch) }
    }
    pub fn set_pitch_semi_tones(&mut self, new_pitch: ::std::os::raw::c_int) {
        unsafe { self.0.setPitchSemiTones(new_pitch) }
    }
    pub fn set_pitch_semi_tones1(&mut self, new_pitch: f64) {
        unsafe { self.0.setPitchSemiTones1(new_pitch) }
    }
    pub fn set_channels(&mut self, num_channels: u32) {
        unsafe { self.0.setChannels(num_channels) }
    }
    pub fn set_sample_rate(&mut self, srate: u32) {
        unsafe { self.0.setSampleRate(srate) }
    }
    pub fn get_input_output_sample_ratio(&mut self) -> f64 {
        unsafe { self.0.getInputOutputSampleRatio() }
    }
    pub fn flush(&mut self) {
        unsafe { self.0.flush() }
    }

    //TODO: Check for validity

    pub fn set_setting(&mut self, setting_id: i32, value: i32) -> bool {
        unsafe { self.0.setSetting(setting_id, value) }
    }
    pub fn get_setting(&self, setting_id: i32) -> i32 {
        unsafe { self.0.getSetting(setting_id) }
    }
}
