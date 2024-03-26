/* Sonic library
   Copyright 2010, 2011
   Bill Cox
   This file is part of the Sonic Library.

   This file is licensed under the Apache 2.0 license.
*/
// package sonic;



const SONIC_MIN_PITCH: i32 = 65;

 const SONIC_MAX_PITCH: i32 = 400;

// This is used to down-sample some inputs to improve speed
 const SONIC_AMDF_FREQ: i32 = 4000;

// The number of points to use in the sinc FIR filter for resampling.
 const SINC_FILTER_POINTS: i32 = 12;

 const SINC_TABLE_SIZE: i32 = 601;

// Lookup table for windowed sinc function of SINC_FILTER_POINTS points.
// The code to generate this is in the header comment of sonic.c.
 const SINC_TABLE: Vec<i16> = vec![0, 0, 0, 0, 0, 0, 0, -1, -1, -2, -2, -3, -4, -6, -7, -9, -10, -12, -14, -17, -19, -21, -24, -26, -29, -32, -34, -37, -40, -42, -44, -47, -48, -50, -51, -52, -53, -53, -53, -52, -50, -48, -46, -43, -39, -34, -29, -22, -16, -8, 0, 9, 19, 29, 41, 53, 65, 79, 92, 107, 121, 137, 152, 168, 184, 200, 215, 231, 247, 262, 276, 291, 304, 317, 328, 339, 348, 357, 363, 369, 372, 374, 375, 373, 369, 363, 355, 345, 332, 318, 300, 281, 259, 234, 208, 178, 147, 113, 77, 39, 0, -41, -85, -130, -177, -225, -274, -324, -375, -426, -478, -530, -581, -632, -682, -731, -779, -825, -870, -912, -951, -989, -1023, -1053, -1080, -1104, -1123, -1138, -1149, -1154, -1155, -1151, -1141, -1125, -1105, -1078, -1046, -1007, -963, -913, -857, -796, -728, -655, -576, -492, -403, -309, -210, -107, 0, 111, 225, 342, 462, 584, 708, 833, 958, 1084, 1209, 1333, 1455, 1575, 1693, 1807, 1916, 2022, 2122, 2216, 2304, 2384, 2457, 2522, 2579, 2625, 2663, 2689, 2706, 2711, 2705, 2687, 2657, 2614, 2559, 2491, 2411, 2317, 2211, 2092, 1960, 1815, 1658, 1489, 1308, 1115, 912, 698, 474, 241, 0, -249, -506, -769, -1037, -1310, -1586, -1864, -2144, -2424, -2703, -2980, -3254, -3523, -3787, -4043, -4291, -4529, -4757, -4972, -5174, -5360, -5531, -5685, -5819, -5935, -6029, -6101, -6150, -6175, -6175, -6149, -6096, -6015, -5905, -5767, -5599, -5401, -5172, -4912, -4621, -4298, -3944, -3558, -3141, -2693, -2214, -1705, -1166, -597, 0, 625, 1277, 1955, 2658, 3386, 4135, 4906, 5697, 6506, 7332, 8173, 9027, 9893, 10769, 11654, 12544, 13439, 14335, 15232, 16128, 17019, 17904, 18782, 19649, 20504, 21345, 22170, 22977, 23763, 24527, 25268, 25982, 26669, 27327, 27953, 28547, 29107, 29632, 30119, 30569, 30979, 31349, 31678, 31964, 32208, 32408, 32565, 32677, 32744, 32767, 32744, 32677, 32565, 32408, 32208, 31964, 31678, 31349, 30979, 30569, 30119, 29632, 29107, 28547, 27953, 27327, 26669, 25982, 25268, 24527, 23763, 22977, 22170, 21345, 20504, 19649, 18782, 17904, 17019, 16128, 15232, 14335, 13439, 12544, 11654, 10769, 9893, 9027, 8173, 7332, 6506, 5697, 4906, 4135, 3386, 2658, 1955, 1277, 625, 0, -597, -1166, -1705, -2214, -2693, -3141, -3558, -3944, -4298, -4621, -4912, -5172, -5401, -5599, -5767, -5905, -6015, -6096, -6149, -6175, -6175, -6150, -6101, -6029, -5935, -5819, -5685, -5531, -5360, -5174, -4972, -4757, -4529, -4291, -4043, -3787, -3523, -3254, -2980, -2703, -2424, -2144, -1864, -1586, -1310, -1037, -769, -506, -249, 0, 241, 474, 698, 912, 1115, 1308, 1489, 1658, 1815, 1960, 2092, 2211, 2317, 2411, 2491, 2559, 2614, 2657, 2687, 2705, 2711, 2706, 2689, 2663, 2625, 2579, 2522, 2457, 2384, 2304, 2216, 2122, 2022, 1916, 1807, 1693, 1575, 1455, 1333, 1209, 1084, 958, 833, 708, 584, 462, 342, 225, 111, 0, -107, -210, -309, -403, -492, -576, -655, -728, -796, -857, -913, -963, -1007, -1046, -1078, -1105, -1125, -1141, -1151, -1155, -1154, -1149, -1138, -1123, -1104, -1080, -1053, -1023, -989, -951, -912, -870, -825, -779, -731, -682, -632, -581, -530, -478, -426, -375, -324, -274, -225, -177, -130, -85, -41, 0, 39, 77, 113, 147, 178, 208, 234, 259, 281, 300, 318, 332, 345, 355, 363, 369, 373, 375, 374, 372, 369, 363, 357, 348, 339, 328, 317, 304, 291, 276, 262, 247, 231, 215, 200, 184, 168, 152, 137, 121, 107, 92, 79, 65, 53, 41, 29, 19, 9, 0, -8, -16, -22, -29, -34, -39, -43, -46, -48, -50, -52, -53, -53, -53, -52, -51, -50, -48, -47, -44, -42, -40, -37, -34, -32, -29, -26, -24, -21, -19, -17, -14, -12, -10, -9, -7, -6, -4, -3, -2, -2, -1, -1, 0, 0, 0, 0, 0, 0, 0, ]
;


pub struct Sonic {
     input_buffer: Vec<i16>,
     output_buffer: Vec<i16>,
     pitch_buffer: Vec<i16>,
     down_sample_buffer: Vec<i16>,
     speed: f32,
     volume: f32,
     pitch: f32,
     rate: f32,
     old_rate_position: usize,
     new_rate_position: usize,
     use_chord_pitch: bool,
     quality: i32,
     num_channels: i32,
     input_buffer_size: usize,
     pitch_buffer_size: usize,
     output_buffer_size: usize,
     num_input_samples: usize,
     num_output_samples: usize,
     num_pitch_samples: usize,
     min_period: i32,
     max_period: i32,
     max_required: i32,
     remaining_input_to_copy: i32,
     sample_rate: i32,
     prev_period: i32,
     prev_min_diff: i32,
     min_diff: i32,
     max_diff: i32,
}

impl Sonic {

    // Resize the array.
    fn  resize(old_array: &[i16],  new_length: usize) -> Vec<i16>  {
        // let new_length = new_length*self.num_channels;
        
        let mut new_array = vec![0; new_length ];
        let length = old_array.len().min(new_length );
        new_array[..length].copy_from_slice(&old_array[..length]);
        new_array
    }

    // Move samples from one array to another.  May move samples down within an array, but not up.
    fn  move_samples_internal(dest: &mut[i16],  dest_pos: usize,  source: &[i16],  source_pos: usize,  num_samples: usize)   {
        
        dest[dest_pos ..(dest_pos  + num_samples )]
        .copy_from_slice(&source[source_pos ..(source_pos  + num_samples )]);
    }

    // Scale the samples by the factor.
    fn scale_samples_internal(&self,samples: &mut [i16], position: usize, num_samples: usize, volume: f32) {
        let fixed_point_volume = (volume * 4096.0) as i32;
        let start = position * self.num_channels as usize;
        let stop = start + num_samples * self.num_channels as usize;
        for x_sample in start..stop {
            let value = (samples[x_sample] * fixed_point_volume as i16) >> 12;
            samples[x_sample] = value.max(-32767).min(32767) as i16;
        }

    }

    // Get the speed of the stream.
    pub fn  get_speed(&self) -> f32  {
        return self.speed;
    }

    // Set the speed of the stream.
    pub fn  set_speed(&self,  speed: f32)   {
        self.speed = speed;
    }

    // Get the pitch of the stream.
    pub fn  get_pitch(&self) -> f32  {
        return self.pitch;
    }

    // Set the pitch of the stream.
    pub fn  set_pitch(&self,  pitch: f32)   {
        self.pitch = pitch;
    }

    // Get the rate of the stream.
    pub fn  get_rate(&self) -> f32  {
        return self.rate;
    }

    // Set the playback rate of the stream. This scales pitch and speed at the same time.
    pub fn  set_rate(&mut self,  rate: f32)   {
        self.rate = rate;
        self.old_rate_position= 0;
        self.new_rate_position= 0;
    }

    // Get the vocal chord pitch setting.
    pub fn  get_chord_pitch(&self) -> bool  {
        return self.use_chord_pitch;
    }

    // Set the vocal chord mode for pitch computation.  Default is off.
    pub fn  set_chord_pitch(&mut self,  use_chord_pitch: bool)   {
        self.use_chord_pitch= use_chord_pitch;
    }

    // Get the quality setting.
    pub fn  get_quality(&self) -> i32  {
        return self.quality;
    }

    // Set the "quality".  Default 0 is virtually as good as 1, but very much faster.
    pub fn  set_quality(&self,  quality: i32)   {
        self.quality = quality;
    }

    // Get the scaling factor of the stream.
    pub fn  get_volume(&self) -> f32  {
        return self.volume;
    }

    // Set the scaling factor of the stream.
    pub fn  set_volume(&self,  volume: f32)   {
        self.volume = volume;
    }

    
    fn resize_input_buffer(&mut self, num_samples: usize) {
        let new_length = self.num_input_samples + num_samples;
        if new_length > self.input_buffer_size {
            self.input_buffer_size += (self.input_buffer_size >> 1) + num_samples;
            self.input_buffer
                .resize(self.input_buffer_size * self.num_channels as usize, 0);
        }
    }

    fn resize_output_buffer(&mut self, num_samples: usize) {
        let new_length = self.num_output_samples + num_samples;
        if new_length > self.output_buffer_size {
            self.output_buffer_size += (self.output_buffer_size >> 1) + num_samples;
            self.output_buffer
                .resize(self.output_buffer_size * self.num_channels as usize, 0);
        }
    }

    fn resize_pitch_buffer(&mut self, num_samples: usize) {
        let new_length = self.num_pitch_samples + num_samples;
        if new_length > self.pitch_buffer_size {
            self.pitch_buffer_size += (self.pitch_buffer_size >> 1) + num_samples;
            self.pitch_buffer
                .resize(self.pitch_buffer_size * self.num_channels as usize, 0);
        }
    }

    fn resize_down_sample_buffer(&mut self, num_samples: usize) {
        let new_length = self.max_required as usize / num_samples;
        self.down_sample_buffer.resize(new_length, 0);
    }

    fn move_samples_input(&mut self, dest_pos: usize, source_pos: usize, num_samples: usize) {
        Self::move_samples_internal(
            &mut self.input_buffer,
            dest_pos * self.num_channels as usize,
            &self.input_buffer,
            source_pos * self.num_channels as usize,
            num_samples * self.num_channels as usize,
        );
    }

    
    fn move_samples_output(&mut self, dest_pos: usize, source_pos: usize, num_samples: usize) {
        Self::move_samples_internal(
            &mut self.output_buffer,
            dest_pos * self.num_channels as usize,
            &self.output_buffer,
            source_pos * self.num_channels as usize,
            num_samples * self.num_channels as usize,
        );
    }
    
    fn scale_samples(&mut self, position: usize, num_samples: usize) {
        Self::scale_samples_internal(
            &self,
            &mut self.input_buffer,
            position * self.num_channels as usize,
            num_samples * self.num_channels as usize,
            self.volume,
        );
    }
    
    // Allocate stream buffers.
    fn  allocate_stream_buffers(&self,  sample_rate: i32,  num_channels: i32)   {
        self.min_period = sample_rate / SONIC_MAX_PITCH;
        self.max_period = sample_rate / SONIC_MIN_PITCH;
        self.max_required = 2 * self.max_period;
        self.input_buffer_size = self.max_required as usize;
        self.input_buffer = vec![0; (self.max_required * num_channels) as usize];
        self.output_buffer_size = self.max_required as usize;
        self.output_buffer = vec! [0; (self.max_required * num_channels) as usize];
        self.pitch_buffer_size = self.max_required as usize;
        self.pitch_buffer = vec! [0; (self.max_required * num_channels) as usize];
        self.down_sample_buffer = vec![0; (self.max_required) as usize];
        self.sample_rate= sample_rate;
        self.num_channels= num_channels;
        self.old_rate_position = 0;
        self.new_rate_position = 0;
        self.prev_period = 0;
    }

    // Create a sonic stream.
    pub fn new( sample_rate: i32,  num_channels: i32) -> Self {

        let max_required = 2 * sample_rate / SONIC_MIN_PITCH;
        let input_buffer_size = max_required as usize;
        let output_buffer_size = max_required as usize;
        let pitch_buffer_size = max_required as usize;
        Sonic {
            input_buffer: vec![0; input_buffer_size * num_channels as usize],
            output_buffer: vec![0; output_buffer_size * num_channels as usize],
            pitch_buffer: vec![0; pitch_buffer_size * num_channels as usize],
            down_sample_buffer: vec![0; max_required as usize],
            speed: 1.0,
            volume: 1.0,
            pitch: 1.0,
            rate: 1.0,
            old_rate_position: 0,
            new_rate_position: 0,
            use_chord_pitch: false,
            quality: 0,
            num_channels,
            input_buffer_size,
            pitch_buffer_size,
            output_buffer_size,
            num_input_samples: 0,
            num_output_samples: 0,
            num_pitch_samples: 0,
            min_period: sample_rate / SONIC_MAX_PITCH,
            max_period: sample_rate / SONIC_MIN_PITCH,
            max_required,
            remaining_input_to_copy: 0,
            sample_rate,
            prev_period: 0,
            prev_min_diff: 0,
            min_diff: 0,
            max_diff: 0,
        }
    }

    // Get the sample rate of the stream.
    pub fn  get_sample_rate(&self) -> i32  {
        return self.sample_rate;
    }

    // Set the sample rate of the stream.  This will cause samples buffered in the stream to be lost.
    pub fn  set_sample_rate(&self,  sample_rate: i32)   {
        self.allocate_stream_buffers(sample_rate, self.num_channels);
    }

    // Get the number of channels.
    pub fn  get_num_channels(&self) -> i32  {
        return self.num_channels;
    }

    // Set the num channels of the stream.  This will cause samples buffered in the stream to be lost.
    pub fn  set_num_channels(&self,  num_channels: i32)   {
        self.allocate_stream_buffers(self.sample_rate, num_channels);
    }

    // Enlarge the output buffer if needed.
    fn  enlarge_output_buffer_if_needed(&self,  num_samples: usize)   {
        if self.num_output_samples + num_samples > self.output_buffer_size {
            self.output_buffer_size += (self.output_buffer_size >> 1) + num_samples;
            self.output_buffer = Self::resize(&self.output_buffer, self.output_buffer_size);
        }
    }

    // Enlarge the input buffer if needed.
    fn  enlarge_input_buffer_if_needed(&self,  num_samples: usize)   {
        if self.num_input_samples + num_samples > self.input_buffer_size {
            self.input_buffer_size += (self.input_buffer_size >> 1) + num_samples;
            self.input_buffer = Self::resize(&self.input_buffer, self.input_buffer_size);
        }
    }

    // Add the input samples to the input buffer.
    fn  add_float_samples_to_input_buffer(&self,  samples: &[f32])   {
        
        let num_samples = samples.len() / self.num_channels as usize;
        if num_samples == 0 {
            return;
        }
        self.resize_input_buffer(num_samples);
        let x_buffer = self.num_input_samples * self.num_channels as usize;
        for x_sample in 0..num_samples * self.num_channels as usize {
            self.input_buffer[x_buffer + x_sample] = (samples[x_sample] * 32767.0) as i16;
        }
        self.num_input_samples += num_samples;
    }

    // Add the input samples to the input buffer.
    fn  add_short_samples_to_input_buffer(&self,  samples: &[i16])   {
        let num_samples = samples.len() / self.num_channels as usize;
        if num_samples == 0 {
            return;
        }
        self.resize_input_buffer(num_samples);
        self.move_samples_input(self.num_input_samples, 0, num_samples);
        self.num_input_samples += num_samples;
    }

    // Add the input samples to the input buffer.
    
   fn add_unsigned_byte_samples_to_input_buffer(&mut self, samples: &[u8]) {
        let num_samples = samples.len() / self.num_channels as usize;
        if num_samples == 0 {
            return;
        }
        self.resize_input_buffer(num_samples);
        let x_buffer = self.num_input_samples * self.num_channels as usize;
        for x_sample in 0..num_samples * self.num_channels as usize {
            let sample = (samples[x_sample] as i16) - 128;
            self.input_buffer[x_buffer + x_sample] = sample << 8;
        }
        self.num_input_samples += num_samples;
    }

    
    // Add the input samples to the input buffer.  They must be 16-bit little-endian encoded in a byte array.
    
    fn add_bytes_to_input_buffer(&mut self, in_buffer: &[u8]) {
        let num_bytes = in_buffer.len();
        let num_samples = num_bytes / (2 * self.num_channels as usize);
        self.resize_input_buffer(num_samples);
        let x_buffer = self.num_input_samples * self.num_channels as usize;
        for x_byte in (0..num_bytes).step_by(2) {
            let sample = (in_buffer[x_byte] as i16) | ((in_buffer[x_byte + 1] as i16) << 8);
            self.input_buffer[x_buffer + x_byte / 2] = sample;
        }
        self.num_input_samples += num_samples;
    }
    

    // Remove input samples that we have already processed.
    
    fn remove_input_samples(&mut self, position: usize) {
        let remaining_samples = self.num_input_samples - position;
        self.move_samples_input(0, position, remaining_samples);
        self.num_input_samples = remaining_samples;
    }
    

    // Just copy from the array to the output buffer
    
    fn copy_to_output(&mut self, samples: &[i16], position: usize, num_samples: usize) {
        self.resize_output_buffer(num_samples);
        Self::move_samples_internal(
            &mut self.output_buffer,
            self.num_output_samples,
            samples,
            position,
            num_samples,
        );
        self.num_output_samples += num_samples;
    }

    // Just copy from the input buffer to the output buffer.  Return num samples copied.
    fn copy_input_to_output(&mut self, position: usize) -> usize {
        let num_samples = self.remaining_input_to_copy as usize;
        if num_samples > self.max_required as usize {
            self.remaining_input_to_copy -= self.max_required;
            return self.max_required as usize;
        }
        self.copy_to_output(&self.input_buffer, position, num_samples);
        self.remaining_input_to_copy -= num_samples as i32;
        num_samples
    }
    
    // Read data out of the stream.  Sometimes no data will be available, and zero
    // is returned, which is not an error condition.
    pub fn read_float_from_stream(&mut self, samples: &mut [f32]) -> usize {
        let num_samples = self.num_output_samples;
        let remaining_samples = if num_samples > samples.len() {
            num_samples - samples.len()
        } else {
            0
        };
        if num_samples == 0 {
            return 0;
        }
        for x_sample in 0..num_samples * self.num_channels as usize {
            samples[x_sample] = self.output_buffer[x_sample] as f32 / 32767.0;
        }
        self.move_samples_output( 0, num_samples,remaining_samples);
        self.num_output_samples = remaining_samples;
        num_samples
    }
    
    // pub fn  read_float_from_stream(&self,  samples: f32,  max_samples: i32) -> i32  {
        
    //      let num_samples: i32 = self.num_output_samples;
    //      let remaining_samples: i32 = 0;
    //     if num_samples == 0 {
    //         return 0;
    //     }
    //     if num_samples > max_samples {
    //         remaining_samples = num_samples - max_samples;
    //         num_samples = max_samples;
    //     }
    //      {
    //          let x_sample: i32 = 0;
    //         while x_sample < num_samples * self.num_channels {
    //             {
    //                 samples[x_sample] = (self.output_buffer[x_sample]) / 32767.0f;
    //             }
    //             x_sample += 1;
    //          }
    //      }

    //     self.move(self.output_buffer, 0, self.output_buffer, num_samples, remaining_samples);
    //     self.num_output_samples = remaining_samples;
    //     return num_samples;
    // }

    // Read short data out of the stream.  Sometimes no data will be available, and zero
    // is returned, which is not an error condition.
     pub fn read_short_from_stream(&mut self, samples: &mut [i16]) -> usize {
            let num_samples = self.num_output_samples;
            let remaining_samples = if num_samples > samples.len() {
                num_samples - samples.len()
            } else {
                0
            };
            if num_samples == 0 {
                return 0;
            }
            Self::move_samples_internal(samples, 0, &self.output_buffer, 0, num_samples);
            self.move_samples_output(0, num_samples,remaining_samples);
            self.num_output_samples = remaining_samples;
            num_samples
        }

    
    
    // pub fn  read_short_from_stream(&self,  samples: i16,  max_samples: i32) -> i32  {
    //      let num_samples: i32 = self.num_output_samples;
    //      let remaining_samples: i32 = 0;
    //     if num_samples == 0 {
    //         return 0;
    //     }
    //     if num_samples > max_samples {
    //         remaining_samples = num_samples - max_samples;
    //         num_samples = max_samples;
    //     }
    //     self.move(samples, 0, self.output_buffer, 0, num_samples);
    //     self.move(self.output_buffer, 0, self.output_buffer, num_samples, remaining_samples);
    //     self.num_output_samples = remaining_samples;
    //     return num_samples;
    // }

    // Read unsigned byte data out of the stream.  Sometimes no data will be available, and zero
    // is returned, which is not an error condition.
    pub fn read_unsigned_byte_from_stream(&mut self, samples: &mut [u8]) -> usize {
        let num_samples = self.num_output_samples;
        let remaining_samples = if num_samples > samples.len() {
            num_samples - samples.len()
        } else {
            0
        };
        if num_samples == 0 {
            return 0;
        }
        for x_sample in 0..num_samples * self.num_channels as usize {
            samples[x_sample] = (self.output_buffer[x_sample] >> 8) as u8 + 128;
        }
        self.move_samples_output( 0, num_samples,remaining_samples);
        self.num_output_samples = remaining_samples;
        num_samples
    }
    
    

    // Read unsigned byte data out of the stream.  Sometimes no data will be available, and zero
    // is returned, which is not an error condition.
 pub fn read_bytes_from_stream(&mut self, out_buffer: &mut [u8]) -> usize {
        let max_samples = out_buffer.len() / (2 * self.num_channels as usize);
        let num_samples = self.num_output_samples;
        let remaining_samples = if num_samples > max_samples {
            num_samples - max_samples
        } else {
            0
        };
        if num_samples == 0 || max_samples == 0 {
            return 0;
        }
        for x_sample in 0..num_samples * self.num_channels as usize {
            let sample = self.output_buffer[x_sample];
            out_buffer[x_sample * 2] = (sample & 0xff) as u8;
            out_buffer[x_sample * 2 + 1] = (sample >> 8) as u8;
        }
        self.move_samples_output(0, num_samples,remaining_samples);
        self.num_output_samples = remaining_samples;
        2 * num_samples * self.num_channels as usize
    }

    

    // Force the sonic stream to generate output using whatever data it currently
    // has.  No extra delay will be added to the output, but flushing in the middle of
    // words could introduce distortion.
 pub fn flush_stream(&mut self) {
        let remaining_samples = self.num_input_samples;
        let s = self.speed / self.pitch;
        let r = self.rate * self.pitch;
        let expected_output_samples = self.num_output_samples
            + ((remaining_samples as f32 / s + self.num_pitch_samples as f32) / r + 0.5) as usize;

        self.resize_input_buffer(remaining_samples + 2 * self.max_required as usize);
        for x_sample in 0..2 * self.max_required as usize * self.num_channels as usize {
            self.input_buffer[remaining_samples * self.num_channels as usize + x_sample] = 0;
        }
        self.num_input_samples += 2 * self.max_required as usize;
        self.write_short_to_stream(&[0]);

        if self.num_output_samples > expected_output_samples {
            self.num_output_samples = expected_output_samples;
        }

        self.num_input_samples = 0;
        self.remaining_input_to_copy = 0;
        self.num_pitch_samples = 0;
    }


    // Return the number of samples in the output buffer
    pub fn  samples_available(&self) -> usize{
        self.num_output_samples
    }

    // If skip is greater than one, average skip samples together and write them to
    // the down-sample buffer.  If numChannels is greater than one, mix the channels
    // together as we down sample.
    
    fn down_sample_input(&mut self, samples: &[i16], position: usize, skip: usize) {
        let num_samples = self.max_required as usize / skip;
        let samples_per_value = self.num_channels * skip as i32;
        let position = position * self.num_channels as usize;
        for i in 0..num_samples {
            let mut value = 0;
            for j in 0..samples_per_value {
                value += samples[position + i * samples_per_value as usize + j as usize];
            }
            value /= samples_per_value as i16;
            self.down_sample_buffer[i] = value;
        }
    }
    

    // Find the best frequency match in the range, and given a sample skip multiple.
    // For now, just find the pitch of the first channel.
     fn find_pitch_period_in_range(
            &mut self,
            samples: &[i16],
            position: usize,
            min_period: i32,
            max_period: i32,
        ) -> i32 {
            let mut best_period = 0;
            let mut worst_pitch = 0;
            let mut worst_pitch_diff = 0;
            let mut period = min_period;
            while period <= max_period {
                let pitch_diff = self.calculate_pitch_diff(samples, position, period);
                if pitch_diff < 0 {
                    break;
                }
                if pitch_diff < worst_pitch_diff || worst_pitch_diff == 0 {
                    worst_pitch_diff = pitch_diff;
                    worst_pitch = period;
                }
                if pitch_diff < self.quality {
                    best_period = period;
                    if pitch_diff == 0 {
                        break;
                    }
                }
                period += 1;
            }
            if best_period == 0 {
                best_period = worst_pitch;
            }
            best_period
        }

    fn calculate_pitch_diff(&mut self, samples: &[i16], position: usize, period: i32) -> i32 {
        let mut diff = 0;
        let mut i = 0;
        while i < period {
            let j = i as usize;
            let k = (i + period) as usize;
            diff += (samples[position + j] - samples[position + k]).abs();
            i += 1;
        }
        diff as i32
    }
    
    // At abrupt ends of voiced words, we can have pitch periods that are better
    // approximated by the previous pitch period estimate.  Try to detect this case.
    
    fn prev_period_better(
        min_diff: i32,
        max_diff: i32,
        prefer_new_period: bool,
        prev_min_diff: i32,
        prev_period: i32,
    ) -> bool {
        if min_diff == 0 || prev_period == 0 {
            return false;
        }
        if prefer_new_period {
            if max_diff > min_diff * 3 {
                return false;
            }
            if min_diff * 2 <= prev_min_diff * 3 {
                return false;
            }
        } else {
            if min_diff <= prev_min_diff {
                return false;
            }
        }
        true
    }

    // Find the pitch period.  This is a critical step, and we may have to try
    // multiple ways to get a good answer.  This version uses AMDF.  To improve
    // speed, we down sample by an integer factor get in the 11KHz range, and then
    // do it again with a narrower frequency range without down sampling
    
    fn find_pitch_period(
            &self,
            samples: &[i16],
            position: usize,
            prefer_new_period: bool,
            sample_rate: i32,
            sonic_amdf_freq: i32,
            quality: i32,
            num_channels: i32,
            min_period: i32,
            max_period: i32,
            prev_min_diff: i32,
            prev_period: i32,
        ) -> i32 {
            let period: i32;
            let ret_period: i32;
            let skip = if sample_rate > sonic_amdf_freq && quality == 0 {
                sample_rate / sonic_amdf_freq
            } else {
                1
            };
            if num_channels == 1 && skip == 1 {
                period = self.find_pitch_period_in_range(
                    samples,
                    position,
                    min_period,
                    max_period,
                    // sample_rate,
                    // sonic_amdf_freq,
                    // quality,
                );
            } else {
                let down_sample_buffer = self.down_sample_input(
                    samples,
                    position,
                    skip as usize,
                    // sample_rate,
                    // sonic_amdf_freq,
                    // quality,
                );
                period = self.find_pitch_period_in_range(
                    &down_sample_buffer,
                    0,
                    min_period / skip,
                    max_period / skip,
                    // sample_rate,
                    // sonic_amdf_freq,
                    // quality,
                );
                if skip != 1 {
                    let period = period * skip;
                    let min_p = period - (skip << 2);
                    let max_p = period + (skip << 2);
                    let min_p = if min_p < min_period {
                        min_period
                    } else {
                        min_p
                    };
                    let max_p = if max_p > max_period {
                        max_period
                    } else {
                        max_p
                    };
                    if num_channels == 1 {
                        period = find_pitch_period_in_range(
                            samples,
                            position,
                            min_p,
                            max_p,
                            sample_rate,
                            sonic_amdf_freq,
                            quality,
                        );
                    } else {
                        let down_sample_buffer = down_sample_input(samples, position, 1);
                        period = find_pitch_period_in_range(
                            &down_sample_buffer,
                            0,
                            min_p,
                            max_p,
                            sample_rate,
                            sonic_amdf_freq,
                            quality,
                        );
                    }
                }
            }
            if prev_period_better(
                min_diff,
                max_diff,
                prefer_new_period,
                prev_min_diff,
                prev_period,
            ) {
                ret_period = prev_period;
            } else {
                ret_period = period;
            }
            ret_period
        }   
    //  fn  find_pitch_period(&self,  samples: i16,  position: i32,  prefer_new_period: bool) -> i32  {
    //     k
    //      let mut period: i32, let ret_period: i32;
    //      let mut skip: i32 = 1;
    //     if self.sample_rate > SONIC_AMDF_FREQ && self.quality == 0 {
    //         skip = self.sample_rate / SONIC_AMDF_FREQ;
    //     }
    //     if self.num_channels == 1 && skip == 1 {
    //         period = self.find_pitch_period_in_range(samples, position, self.min_period, self.max_period);
    //     } else {
    //         self.down_sample_input(samples, position, skip);
    //         period = self.find_pitch_period_in_range(self.down_sample_buffer, 0, self.min_period / skip, self.max_period / skip);
    //         if skip != 1 {
    //             period *= skip;
    //              let min_p: i32 = period - (skip << 2);
    //              let max_p: i32 = period + (skip << 2);
    //             if min_p < self.min_period {
    //                 min_p = self.min_period;
    //             }
    //             if max_p > self.max_period {
    //                 max_p = self.max_period;
    //             }
    //             if self.num_channels == 1 {
    //                 period = self.find_pitch_period_in_range(samples, position, min_p, max_p);
    //             } else {
    //                 self.down_sample_input(samples, position, 1);
    //                 period = self.find_pitch_period_in_range(self.down_sample_buffer, 0, min_p, max_p);
    //             }
    //         }
    //     }
    //     if self.prev_period_better(self.min_diff, self.max_diff, prefer_new_period) {
    //         ret_period = self.prev_period;
    //     } else {
    //         ret_period = period;
    //     }
    //     self.prev_min_diff = self.min_diff;
    //     self.prev_period = period;
    //     return ret_period;
    // }

    // Overlap two sound segments, ramp the volume of one down, while ramping the
    // other one from zero up, and add them, storing the result at the output.
    fn  overlap_add(&self,  num_samples: i32,  num_channels: i32,  out: i16,  out_pos: i32,  ramp_down: i16,  ramp_down_pos: i32,  ramp_up: i16,  ramp_up_pos: i32)   {
         {
             let mut i: i32 = 0;
            while i < num_channels {
                {
                     let mut o: i32 = out_pos * num_channels + i;
                     let mut u: i32 = ramp_up_pos * num_channels + i;
                     let mut d: i32 = ramp_down_pos * num_channels + i;
                     {
                         let mut t: i32 = 0;
                        while t < num_samples {
                            {
                                out[o] = ((ramp_down[d] * (num_samples - t) + ramp_up[u] * t) / num_samples) as i16;
                                o += num_channels;
                                d += num_channels;
                                u += num_channels;
                            }
                            t += 1;
                         }
                     }

                }
                i += 1;
             }
         }

    }

    // Overlap two sound segments, ramp the volume of one down, while ramping the
    // other one from zero up, and add them, storing the result at the output.
    fn  overlap_add_with_separation(&self,  num_samples: i32,  num_channels: i32,  separation: i32,  out: i16,  out_pos: i32,  ramp_down: i16,  ramp_down_pos: i32,  ramp_up: i16,  ramp_up_pos: i32)   {
         {
             let mut i: i32 = 0;
            while i < num_channels {
                {
                     let mut o: i32 = out_pos * num_channels + i;
                     let mut u: i32 = ramp_up_pos * num_channels + i;
                     let mut d: i32 = ramp_down_pos * num_channels + i;
                     {
                         let mut t: i32 = 0;
                        while t < num_samples + separation {
                            {
                                if t < separation {
                                    out[o] = (ramp_down[d] * (num_samples - t) / num_samples) as i16;
                                    d += num_channels;
                                } else if t < num_samples {
                                    out[o] = ((ramp_down[d] * (num_samples - t) + ramp_up[u] * (t - separation)) / num_samples) as i16;
                                    d += num_channels;
                                    u += num_channels;
                                } else {
                                    out[o] = (ramp_up[u] * (t - separation) / num_samples) as i16;
                                    u += num_channels;
                                }
                                o += num_channels;
                            }
                            t += 1;
                         }
                     }

                }
                i += 1;
             }
         }

    }

    // Just move the new samples in the output buffer to the pitch buffer
    fn  move_new_samples_to_pitch_buffer(&self,  original_num_output_samples: i32)   {
         let num_samples: i32 = self.num_output_samples - original_num_output_samples;
        if self.num_pitch_samples + num_samples > self.pitch_buffer_size {
            self.pitch_buffer_size += (self.pitch_buffer_size >> 1) + num_samples;
            self.pitch_buffer = self.resize(self.pitch_buffer, self.pitch_buffer_size);
        }
        self.move(self.pitch_buffer, self.num_pitch_samples, self.output_buffer, original_num_output_samples, num_samples);
        self.num_output_samples = original_num_output_samples;
        self.num_pitch_samples += num_samples;
    }

    // Remove processed samples from the pitch buffer.
    fn  remove_pitch_samples(&self,  num_samples: i32)   {
        if num_samples == 0 {
            return;
        }
        self.move(self.pitch_buffer, 0, self.pitch_buffer, num_samples, self.num_pitch_samples - num_samples);
        self.num_pitch_samples -= num_samples;
    }

    // Change the pitch.  The latency this introduces could be reduced by looking at
    // past samples to determine pitch, rather than future.
    fn  adjust_pitch(&self,  original_num_output_samples: i32)   {
         let mut period: i32, let new_period: i32, let mut separation: i32;
         let mut position: i32 = 0;
        if self.num_output_samples == original_num_output_samples {
            return;
        }
        self.move_new_samples_to_pitch_buffer(original_num_output_samples);
        while self.num_pitch_samples - position >= self.max_required {
            period = self.find_pitch_period(self.pitch_buffer, position, false);
            new_period = (period / self.pitch) as i32;
            self.enlarge_output_buffer_if_needed(new_period);
            if self.pitch >= 1.0f {
                self.overlap_add(new_period, self.num_channels, self.output_buffer, self.num_output_samples, self.pitch_buffer, position, self.pitch_buffer, position + period - new_period);
            } else {
                separation = new_period - period;
                self.overlap_add_with_separation(period, self.num_channels, separation, self.output_buffer, self.num_output_samples, self.pitch_buffer, position, self.pitch_buffer, position);
            }
            self.num_output_samples += new_period;
            position += period;
        }
        self.remove_pitch_samples(position);
    }

    // Approximate the sinc function times a Hann window from the sinc table.
    fn  find_sinc_coefficient(&self,  i: i32,  ratio: i32,  width: i32) -> i32  {
         let lobe_points: i32 = (SINC_TABLE_SIZE - 1) / SINC_FILTER_POINTS;
         let left: i32 = i * lobe_points + (ratio * lobe_points) / width;
         let right: i32 = left + 1;
         let position: i32 = i * lobe_points * width + ratio * lobe_points - left * width;
         let left_val: i32 = SINC_TABLE[left];
         let right_val: i32 = SINC_TABLE[right];
        return ((left_val * (width - position) + right_val * position) << 1) / width;
    }

    // Return 1 if value >= 0, else -1.  This represents the sign of value.
    fn  get_sign(&self,  value: i32) -> i32  {
        return  if value >= 0 { 1 } else { -1 };
    }

    // Interpolate the new output sample.
    fn  interpolate(&self,  in: i16, // Index to first sample which already includes channel offset.
     in_pos: i32,  old_sample_rate: i32,  new_sample_rate: i32) -> i16  {
        // Compute N-point sinc FIR-filter here.  Clip rather than overflow.
         let mut i: i32;
         let mut total: i32 = 0;
         let position: i32 = self.new_rate_position * old_sample_rate;
         let left_position: i32 = self.old_rate_position * new_sample_rate;
         let right_position: i32 = (self.old_rate_position + 1) * new_sample_rate;
         let ratio: i32 = right_position - position - 1;
         let width: i32 = right_position - left_position;
         let mut weight: i32, let mut value: i32;
         let old_sign: i32;
         let overflow_count: i32 = 0;
         {
            i = 0;
            while i < SINC_FILTER_POINTS {
                {
                    weight = self.find_sinc_coefficient(i, ratio, width);
                    /* printf("%u %f\n", i, weight); */
                    value = in[in_pos + i * self.num_channels] * weight;
                    old_sign = self.get_sign(total);
                    total += value;
                    if old_sign != self.get_sign(total) && self.get_sign(value) == old_sign {
                        /* We must have overflowed.  This can happen with a sinc filter. */
                        overflow_count += old_sign;
                    }
                }
                i += 1;
             }
         }

        /* It is better to clip than to wrap if there was a overflow. */
        if overflow_count > 0 {
            return Short::MAX_VALUE;
        } else if overflow_count < 0 {
            return Short::MIN_VALUE;
        }
        return (total >> 16) as i16;
    }

    // Change the rate.
    fn  adjust_rate(&self,  rate: f32,  original_num_output_samples: i32)   {
         let new_sample_rate: i32 = (self.sample_rate / rate) as i32;
         let old_sample_rate: i32 = self.sample_rate;
         let mut position: i32;
         const N: i32 = SINC_FILTER_POINTS;
        // Set these values to help with the integer math
        while new_sample_rate > (1 << 14) || old_sample_rate > (1 << 14) {
            new_sample_rate >>= 1;
            old_sample_rate >>= 1;
        }
        if self.num_output_samples == original_num_output_samples {
            return;
        }
        self.move_new_samples_to_pitch_buffer(original_num_output_samples);
        // Leave at least N pitch samples in the buffer
         {
            position = 0;
            while position < self.num_pitch_samples - N {
                {
                    while (self.old_rate_position + 1) * new_sample_rate > self.new_rate_position * old_sample_rate {
                        self.enlarge_output_buffer_if_needed(1);
                         {
                             let mut i: i32 = 0;
                            while i < self.num_channels {
                                {
                                    self.output_buffer[self.num_output_samples * self.num_channels + i] = self.interpolate(self.pitch_buffer, position * self.num_channels + i, old_sample_rate, new_sample_rate);
                                }
                                i += 1;
                             }
                         }

                        self.new_rate_position += 1;
                        self.num_output_samples += 1;
                    }
                    self.old_rate_position += 1;
                    if self.old_rate_position == old_sample_rate {
                        self.old_rate_position = 0;
                        if self.new_rate_position != new_sample_rate {
                            System::out::printf("Assertion failed: newRatePosition != newSampleRate\n");
                            assert!( false);
                        }
                        self.new_rate_position = 0;
                    }
                }
                position += 1;
             }
         }

        self.remove_pitch_samples(position);
    }

    // Skip over a pitch period, and copy period/speed samples to the output
    fn  skip_pitch_period(&self,  samples: i16,  position: i32,  speed: f32,  period: i32) -> i32  {
         let new_samples: i32;
        if speed >= 2.0f {
            new_samples = (period / (speed - 1.0f)) as i32;
        } else {
            new_samples = period;
            self.remaining_input_to_copy = (period * (2.0f - speed) / (speed - 1.0f)) as i32;
        }
        self.enlarge_output_buffer_if_needed(new_samples);
        self.overlap_add(new_samples, self.num_channels, self.output_buffer, self.num_output_samples, samples, position, samples, position + period);
        self.num_output_samples += new_samples;
        return new_samples;
    }

    // Insert a pitch period, and determine how much input to copy directly.
    fn  insert_pitch_period(&self,  samples: i16,  position: i32,  speed: f32,  period: i32) -> i32  {
         let new_samples: i32;
        if speed < 0.5f {
            new_samples = (period * speed / (1.0f - speed)) as i32;
        } else {
            new_samples = period;
            self.remaining_input_to_copy = (period * (2.0f * speed - 1.0f) / (1.0f - speed)) as i32;
        }
        self.enlarge_output_buffer_if_needed(period + new_samples);
        self.move(self.output_buffer, self.num_output_samples, samples, position, period);
        self.overlap_add(new_samples, self.num_channels, self.output_buffer, self.num_output_samples + period, samples, position + period, samples, position);
        self.num_output_samples += period + new_samples;
        return new_samples;
    }

    // Resample as many pitch periods as we have buffered on the input.  Return 0 if
    // we fail to resize an input or output buffer.  Also scale the output by the volume.
    fn  change_speed(&self,  speed: f32)   {
         let num_samples: i32 = self.num_input_samples;
         let mut position: i32 = 0, let mut period: i32, let new_samples: i32;
        if self.num_input_samples < self.max_required {
            return;
        }
        loop { {
            if self.remaining_input_to_copy > 0 {
                new_samples = self.copy_input_to_output(position);
                position += new_samples;
            } else {
                period = self.find_pitch_period(self.input_buffer, position, true);
                if speed > 1.0 {
                    new_samples = self.skip_pitch_period(self.input_buffer, position, speed, period);
                    position += period + new_samples;
                } else {
                    new_samples = self.insert_pitch_period(self.input_buffer, position, speed, period);
                    position += new_samples;
                }
            }
        }if !(position + self.max_required <= num_samples) break;}
        self.remove_input_samples(position);
    }

    // Resample as many pitch periods as we have buffered on the input.  Scale the output by the volume.
    fn  process_stream_input(&self)   {
         let original_num_output_samples: i32 = self.num_output_samples;
         let s: f32 = self.speed / self.pitch;
         let mut r: f32 = self.rate;
        if !self.use_chord_pitch {
            r *= self.pitch;
        }
        if s > 1.00001 || s < 0.99999 {
            self.change_speed(s);
        } else {
            self.copy_to_output(self.input_buffer, 0, self.num_input_samples);
            self.num_input_samples = 0;
        }
        if self.use_chord_pitch {
            if self.pitch != 1.0f {
                self.adjust_pitch(original_num_output_samples);
            }
        } else if r != 1.0f {
            self.adjust_rate(r, original_num_output_samples);
        }
        if self.volume != 1.0f {
            // Adjust output volume.
            self.scale_samples(self.output_buffer, original_num_output_samples, self.num_output_samples - original_num_output_samples, self.volume);
        }
    }

    // Write floating point data to the input buffer and process it.
    pub fn  write_float_to_stream(&self,  samples: &[f32] )   {
        self.add_float_samples_to_input_buffer(samples );
        self.process_stream_input();
    }

    // Write the data to the input stream, and process it.
    pub fn  write_short_to_stream(&self,  samples: &[i16] )   {
        self.add_short_samples_to_input_buffer(samples);
        self.process_stream_input();
    }

    // Simple wrapper around sonicWriteFloatToStream that does the unsigned byte to short
    // conversion for you.
    pub fn  write_unsigned_byte_to_stream(&self,  samples: &[u8])   {
        self.add_unsigned_byte_samples_to_input_buffer(samples);
        self.process_stream_input();
    }

    // Simple wrapper around sonicWriteBytesToStream that does the byte to 16-bit LE conversion.
    pub fn  write_bytes_to_stream(&self,  in_buffer: &[u8]  )   {
        self.add_bytes_to_input_buffer(in_buffer);
        self.process_stream_input();
    }

    // This is a non-stream oriented interface to just change the speed of a sound sample
    fn change_float_speed(
        samples: &mut [f32],
        num_samples: usize,
        speed: f32,
        pitch: f32,
        rate: f32,
        volume: f32,
        use_chord_pitch: bool,
        sample_rate: i32,
        num_channels: i32,
    ) -> usize {
        let mut stream = Sonic::new(sample_rate, num_channels);
        stream.set_speed(speed);
        stream.set_pitch(pitch);
        stream.set_rate(rate);
        stream.set_volume(volume);
        stream.set_chord_pitch(use_chord_pitch);
        stream.write_float_to_stream(samples);
        stream.flush_stream();
        let num_samples = stream.samples_available();
        stream.read_float_from_stream(samples);
        num_samples
    }
        
        
    // pub fn  change_float_speed( samples: f32,  num_samples: i32,  speed: f32,  pitch: f32,  rate: f32,  volume: f32,  use_chord_pitch: bool,  sample_rate: i32,  num_channels: i32) -> i32  {
    //      let stream: Sonic = Sonic::new(sample_rate, num_channels);
    //     stream.set_speed(speed);
    //     stream.set_pitch(pitch);
    //     stream.set_rate(rate);
    //     stream.set_volume(volume);
    //     stream.set_chord_pitch(use_chord_pitch);
    //     stream.write_float_to_stream(&samples);
    //     stream.flush_stream();
    //     num_samples = stream.samples_available();
    //     stream.read_float_from_stream(samples, num_samples);
    //     return num_samples;
    // }

    /* This is a non-stream oriented interface to just change the speed of a sound sample */
    // pub fn  sonic_change_short_speed(&self,  samples: i16,  num_samples: i32,  speed: f32,  pitch: f32,  rate: f32,  volume: f32,  use_chord_pitch: bool,  sample_rate: i32,  num_channels: i32) -> i32  {
    //      let stream: Sonic = Sonic::new(sample_rate, num_channels);
    //     stream.set_speed(speed);
    //     stream.set_pitch(pitch);
    //     stream.set_rate(rate);
    //     stream.set_volume(volume);
    //     stream.set_chord_pitch(use_chord_pitch);
    //     stream.write_short_to_stream(samples, num_samples);
    //     stream.flush_stream();
    //     num_samples = stream.samples_available();
    //     stream.read_short_from_stream(samples, num_samples);
    //     return num_samples;
    // }
        
    fn sonic_change_short_speed(
        samples: &mut [i16],
        num_samples: usize,
        speed: f32,
        pitch: f32,
        rate: f32,
        volume: f32,
        use_chord_pitch: bool,
        sample_rate: i32,
        num_channels: i32,
    ) -> usize {
        let mut stream = Sonic::new(sample_rate, num_channels);
        stream.set_speed(speed);
        stream.set_pitch(pitch);
        stream.set_rate(rate);
        stream.set_volume(volume);
        stream.set_chord_pitch(use_chord_pitch);
        stream.write_short_to_stream(samples );
        stream.flush_stream();
        let num_samples = stream.samples_available();
        stream.read_short_from_stream(samples );
        num_samples
    }
}

