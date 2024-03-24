/*
 * scaletempo audio filter
 *
 * scale tempo while maintaining pitch
 * (WSOLA technique with cross correlation)
 * inspired by SoundTouch library by Olli Parviainen
 *
 * basic algorithm
 *   - produce 'stride' output samples per loop
 *   - consume stride*scale input samples per loop
 *
 * to produce smoother transitions between strides, blend next overlap
 * samples from last stride with correlated samples of current input
 *
 * Copyright (c) 2007 Robert Juliano
 *
 * This file is part of mpv.
 *
 * mpv is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * mpv is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with mpv.  If not, see <http://www.gnu.org/licenses/>.
 */
use std::mem;
use std::ptr;
use std::slice;
use std::cmp::min;
use std::f32::consts::PI;

struct FOpts {
    scale_nominal: f32,
    ms_stride: f32,
    ms_search: f32,
    factor_overlap: f32,
    speed_opt: i32,
}

struct mp_aframe {
    av_frame: *mut AVFrame,
    // We support channel layouts different from AVFrame channel masks
    chmap: mp_chmap,
    // We support spdif formats, which are allocated as AV_SAMPLE_FMT_S16.
    format: i32,
    pts: f64,
    speed: f64,
}

struct mp_aframe_pool {
    av_frame: *mut AVFrame,
    element_size: i32,
}

struct PrivData {
    opts: FOpts,
    in_pin: *mut mp_pin,
    cur_format: *mut mp_aframe,
    out_pool: *mut mp_aframe_pool,
    current_pts: f64,
    in_: *mut mp_aframe,
    scale: f32,
    speed: f32,
    frames_stride: i32,
    frames_stride_scaled: f32,
    frames_stride_error: f32,
    bytes_per_frame: i32,
    bytes_stride: i32,
    bytes_queue: i32,
    bytes_queued: i32,
    bytes_to_slide: i32,
    buf_queue: *mut i8,
    samples_overlap: i32,
    samples_standing: i32,
    bytes_overlap: i32,
    bytes_standing: i32,
    buf_overlap: *mut i8,
    table_blend: *mut i8,
    output_overlap: Option<unsafe extern "C" fn(*mut PrivData, *mut i8, i32)>,
    frames_search: i32,
    num_channels: i32,
    buf_pre_corr: *mut i8,
    table_window: *mut i8,
    best_overlap_offset: Option<unsafe extern "C" fn(*mut PrivData) -> i32>,
}

fn reinit(f: *mut mp_filter) -> bool {
    let s = (*(f as *mut PrivData)).opts;
    mp_aframe_reset((*(f as *mut PrivData)).cur_format);
    let srate = mp_aframe_get_rate((*(f as *mut PrivData)).in_) / 1000.0;
    let nch = mp_aframe_get_channels((*(f as *mut PrivData)).in_);
    let format = mp_aframe_get_format((*(f as *mut PrivData)).in_);
    let mut use_int = 0;
    if format == AF_FORMAT_S16 {
        use_int = 1;
    } else if format != AF_FORMAT_FLOAT {
        return false;
    }
    let bps = if use_int != 0 { 2 } else { 4 };
    (*(f as *mut PrivData)).frames_stride = (srate * s.ms_stride) as i32;
    (*(f as *mut PrivData)).bytes_stride = (*(f as *mut PrivData)).frames_stride * bps * nch;
    update_speed(f, (*(f as *mut PrivData)).speed);
    let frames_overlap = (*(f as *mut PrivData)).frames_stride * (s.factor_overlap) as i32;
    if frames_overlap <= 0 {
        (*(f as *mut PrivData)).bytes_standing = (*(f as *mut PrivData)).bytes_stride;
        (*(f as *mut PrivData)).samples_standing = (*(f as *mut PrivData)).bytes_standing / bps;
        (*(f as *mut PrivData)).output_overlap = None;
        (*(f as *mut PrivData)).bytes_overlap = 0;
    } else {
        (*(f as *mut PrivData)).samples_overlap = frames_overlap * nch;
        (*(f as *mut PrivData)).bytes_overlap = frames_overlap * nch * bps;
        (*(f as *mut PrivData)).bytes_standing = (*(f as *mut PrivData)).bytes_stride - (*(f as *mut PrivData)).bytes_overlap;
        (*(f as *mut PrivData)).samples_standing = (*(f as *mut PrivData)).bytes_standing / bps;
        (*(f as *mut PrivData)).buf_overlap = realloc((*(f as *mut PrivData)).buf_overlap, (*(f as *mut PrivData)).bytes_overlap);
        (*(f as *mut PrivData)).table_blend = realloc((*(f as *mut PrivData)).table_blend, (*(f as *mut PrivData)).bytes_overlap * 4);
        if (*(f as *mut PrivData)).buf_overlap.is_null() || (*(f as *mut PrivData)).table_blend.is_null() {
            MP_FATAL(f, "Out of memory\n");
            return false;
        }
        memset((*(f as *mut PrivData)).buf_overlap as *mut _, 0, (*(f as *mut PrivData)).bytes_overlap);
        if use_int != 0 {
            let pb = (*(f as *mut PrivData)).table_blend as *mut i32;
            let mut blend = 0;
            for i in 0..frames_overlap {
                let v = blend / frames_overlap;
                for j in 0..nch {
                    *pb.offset((i * nch + j) as isize) = v;
                }
                blend += 65536;
            }
            (*(f as *mut PrivData)).output_overlap = Some(output_overlap_s16);
        } else {
            let pb = (*(f as *mut PrivData)).table_blend as *mut f32;
            for i in 0..frames_overlap {
                let v = i as f32 / frames_overlap as f32;
                for j in 0..nch {
                    *pb.offset((i * nch + j) as isize) = v;
                }
            }
            (*(f as *mut PrivData)).output_overlap = Some(output_overlap_float);
        }
    }
    (*(f as *mut PrivData)).frames_search = if frames_overlap > 1 { (srate * s.ms_search) as i32 } else { 0 };
    if (*(f as *mut PrivData)).frames_search <= 0 {
        (*(f as *mut PrivData)).best_overlap_offset = None;
    } else {
        if use_int != 0 {
            let t = frames_overlap;
            let n = 8589934588 / (t * t);
            (*(f as *mut PrivData)).buf_pre_corr = realloc((*(f as *mut PrivData)).buf_pre_corr, (*(f as *mut PrivData)).bytes_overlap * 2 + UNROLL_PADDING);
            (*(f as *mut PrivData)).table_window = realloc((*(f as *mut PrivData)).table_window, (*(f as *mut PrivData)).bytes_overlap * 2 - nch * bps * 2);
            if (*(f as *mut PrivData)).buf_pre_corr.is_null() || (*(f as *mut PrivData)).table_window.is_null() {
                MP_FATAL(f, "Out of memory\n");
                return false;
            }
            memset((*(f as *mut PrivData)).buf_pre_corr.offset(((*(f as *mut PrivData)).bytes_overlap * 2) as isize) as *mut _, 0, UNROLL_PADDING);
            let pw = (*(f as *mut PrivData)).table_window as *mut i32;
            for i in 1..frames_overlap {
                let v = (i * (t - i) * n) >> 15;
                for j in 0..nch {
                    *pw.offset((i * nch + j) as isize) = v;
                }
            }
            (*(f as *mut PrivData)).best_overlap_offset = Some(best_overlap_offset_s16);
        } else {
            (*(f as *mut PrivData)).buf_pre_corr = realloc((*(f as *mut PrivData)).buf_pre_corr, (*(f as *mut PrivData)).bytes_overlap);
            (*(f as *mut PrivData)).table_window = realloc((*(f as *mut PrivData)).table_window, (*(f as *mut PrivData)).bytes_overlap - nch * bps);
            if (*(f as *mut PrivData)).buf_pre_corr.is_null() || (*(f as *mut PrivData)).table_window.is_null() {
                MP_FATAL(f, "Out of memory\n");
                return false;
            }
            let pw = (*(f as *mut PrivData)).table_window as *mut f32;
            for i in 1..frames_overlap {
                let v = i as f32 * (frames_overlap - i) as f32;
                for j in 0..nch {
                    *pw.offset((i * nch + j) as isize) = v;
                }
            }
            (*(f as *mut PrivData)).best_overlap_offset = Some(best_overlap_offset_float);
        }
    }
    (*(f as *mut PrivData)).bytes_per_frame = bps * nch;
    (*(f as *mut PrivData)).num_channels = nch;
    (*(f as *mut PrivData)).bytes_queue = ((*(f as *mut PrivData)).frames_search + (*(f as *mut PrivData)).frames_stride + frames_overlap) * bps * nch;
    (*(f as *mut PrivData)).buf_queue = realloc((*(f as *mut PrivData)).buf_queue, (*(f as *mut PrivData)).bytes_queue + UNROLL_PADDING);
    if (*(f as *mut PrivData)).buf_queue.is_null() {
        MP_FATAL(f, "Out of memory\n");
        return false;
    }
    (*(f as *mut PrivData)).bytes_queued = 0;
    (*(f as *mut PrivData)).bytes_to_slide = 0;
    MP_DBG(f, "",
           "%.2f stride_in, %i stride_out, %i standing, ",
           "%i overlap, %i search, %i queue, %s mode\n",
           (*(f as *mut PrivData)).frames_stride_scaled,
           (*(f as *mut PrivData)).bytes_stride / nch / bps,
           (*(f as *mut PrivData)).bytes_standing / nch / bps,
           (*(f as *mut PrivData)).bytes_overlap / nch / bps,
           (*(f as *mut PrivData)).frames_search,
           (*(f as *mut PrivData)).bytes_queue / nch / bps,
           if use_int != 0 { "s16" } else { "float" });
    mp_aframe_config_copy((*(f as *mut PrivData)).cur_format, (*(f as *mut PrivData)).in_);
    true
}

fn fill_queue(s: *mut PrivData) -> bool {
    let bytes_in = if !s.is_null() { mp_aframe_get_size(s.in_) * s.bytes_per_frame } else { 0 };
    let mut offset = 0;
    if s.bytes_to_slide > 0 {
        if s.bytes_to_slide < s.bytes_queued {
            let bytes_move = s.bytes_queued - s.bytes_to_slide;
            memmove(s.buf_queue, s.buf_queue.offset(s.bytes_to_slide as isize), bytes_move);
            s.bytes_to_slide = 0;
            s.bytes_queued = bytes_move;
        } else {
            let mut bytes_skip;
            s.bytes_to_slide -= s.bytes_queued;
            bytes_skip = min(s.bytes_to_slide, bytes_in);
            s.bytes_queued = 0;
            s.bytes_to_slide -= bytes_skip;
            offset += bytes_skip;
            bytes_in -= bytes_skip;
        }
    }
    let bytes_needed = s.bytes_queue - s.bytes_queued;
    assert!(bytes_needed >= 0);
    let bytes_copy = min(bytes_needed, bytes_in);
    if bytes_copy > 0 {
        let planes = mp_aframe_get_data_ro(s.in_);
        memcpy(s.buf_queue.offset(s.bytes_queued as isize), planes[0].offset(offset as isize), bytes_copy);
        s.bytes_queued += bytes_copy;
        offset += bytes_copy;
    }
    if !s.in_.is_null() {
        mp_aframe_skip_samples(s.in_, (offset / s.bytes_per_frame) as i32);
    }
    bytes_needed == 0
}

fn best_overlap_offset_float(s: *mut PrivData) -> i32 {
    let mut best_corr = i32::MIN;
    let mut best_off = 0;
    let pw = s.table_window as *mut f32;
    let po = s.buf_overlap as *mut f32;
    let mut ppc = s.buf_pre_corr as *mut f32;
    for i in 0..s.samples_overlap {
        *ppc = *pw * *po;
        ppc = ppc.offset(1);
        po = po.offset(1);
        pw = pw.offset(1);
    }
    let search_start = (s.buf_queue as *mut f32).offset(s.num_channels as isize);
    for off in 0..s.frames_search {
        let mut corr = 0.0;
        let mut ps = search_start;
        ppc = s.buf_pre_corr as *mut f32;
        for i in 0..s.samples_overlap {
            corr += *ppc * *ps;
            ppc = ppc.offset(1);
            ps = ps.offset(1);
        }
        if corr > best_corr as f32 {
            best_corr = corr as i32;
            best_off = off;
        }
        search_start = search_start.offset(s.num_channels as isize);
    }
    best_off * 4 * s.num_channels
}

fn best_overlap_offset_s16(s: *mut PrivData) -> i32 {
    let mut best_corr = i64::MIN;
    let mut best_off = 0;
    let pw = s.table_window as *mut i32;
    let po = s.buf_overlap as *mut i16;
    let mut ppc = s.buf_pre_corr as *mut i32;
    for i in s.num_channels..s.samples_overlap {
        *ppc = (*pw * *po) >> 15;
        ppc = ppc.offset(1);
        po = po.offset(1);
        pw = pw.offset(1);
    }
    let search_start = (s.buf_queue as *mut i16).offset(s.num_channels as isize);
    for off in 0..s.frames_search {
        let mut corr = 0;
        let mut ps = search_start;
        ppc = s.buf_pre_corr as *mut i32;
        ppc = ppc.offset(s.samples_overlap - s.num_channels as isize);
        ps = ps.offset(s.samples_overlap - s.num_channels as isize);
        let mut i = -(s.samples_overlap - s.num_channels);
        loop {
            corr += ppc.offset(i + 0).read() * ps.offset(i + 0).read() as i64;
            corr += ppc.offset(i + 1).read() * ps.offset(i + 1).read() as i64;
            corr += ppc.offset(i + 2).read() * ps.offset(i + 2).read() as i64;
            corr += ppc.offset(i + 3).read() * ps.offset(i + 3).read() as i64;
            i += 4;
            if i >= 0 {
                break;
            }
        }
        if corr > best_corr {
            best_corr = corr;
            best_off = off;
        }
        search_start = search_start.offset(s.num_channels as isize);
    }
    best_off * 2 * s.num_channels
}

fn output_overlap_float(s: *mut PrivData, buf_out: *mut i8, bytes_off: i32) {
    let pout = buf_out as *mut f32;
    let pb = s.table_blend as *mut f32;
    let po = s.buf_overlap as *mut f32;
    let pin = (s.buf_queue as *mut f32).offset(bytes_off as isize);
    for i in 0..s.samples_overlap {
        *pout.offset(i as isize) = *po.offset(i as isize) - *pb.offset(i as isize) * (*po.offset(i as isize) - *pin.offset(i as isize));
    }
}

fn output_overlap_s16(s: *mut PrivData, buf_out: *mut i8, bytes_off: i32) {
    let pout = buf_out as *mut i16;
    let pb = s.table_blend as *mut i32;
    let po = s.buf_overlap as *mut i16;
    let pin = (s.buf_queue as *mut i16).offset(bytes_off as isize);
    for i in 0..s.samples_overlap {
        *pout.offset(i as isize) = *po.offset(i as isize) - ((*pb.offset(i as isize) * (*po.offset(i as isize) - *pin.offset(i as isize))) >> 16);
    }
}

fn process(f: *mut mp_filter) {
    let s = (*(f as *mut PrivData)).opts;
    if !mp_pin_in_needs_data((*(f as *mut PrivData)).in_pin) {
        return;
    }
    let mut out: *mut mp_aframe = ptr::null_mut();
    let mut drain = false;
    let mut is_eof = false;
    if (*(f as *mut PrivData)).in_.is_null() {
        let mut frame = mp_pin_out_read((*(f as *mut PrivData)).in_pin);
        if frame.type_ == 0 {
            return;
        }
        if frame.type_ != MP_FRAME_AUDIO && frame.type_ != MP_FRAME_EOF {
            MP_ERR(f, "unexpected frame type\n");
            goto error;
        }
        (*(f as *mut PrivData)).in_ = if frame.type_ == MP_FRAME_AUDIO { frame.data } else { ptr::null_mut() };
        is_eof = drain = (*(f as *mut PrivData)).in_.is_null();
        if is_eof && !mp_aframe_config_is_valid((*(f as *mut PrivData)).cur_format) {
            mp_pin_in_write((*(f as *mut PrivData)).ppins[1], MP_EOF_FRAME);
            return;
        }
        if !(*(f as *mut PrivData)).in_.is_null() && !mp_aframe_config_equals((*(f as *mut PrivData)).in_, (*(f as *mut PrivData)).cur_format) {
            if (*(f as *mut PrivData)).bytes_queued > 0 {
                MP_VERBOSE(f, "draining\n");
                mp_pin_out_unread((*(f as *mut PrivData)).in_pin, frame);
                (*(f as *mut PrivData)).in_ = ptr::null_mut();
                drain = true;
            } else {
                if !reinit(f) {
                    MP_ERR(f, "initialization failed\n");
                    goto error;
                }
            }
        }
        if !(*(f as *mut PrivData)).in_.is_null() {
            (*(f as *mut PrivData)).current_pts = mp_aframe_end_pts((*(f as *mut PrivData)).in_);
        }
    }
    if !fill_queue((*(f as *mut PrivData)).in_) && !drain {
        TA_FREEP(&mut (*(f as *mut PrivData)).in_);
        mp_pin_out_request_data_next((*(f as *mut PrivData)).in_pin);
        return;
    }
    let max_out_samples = (*(f as *mut PrivData)).bytes_stride / (*(f as *mut PrivData)).bytes_per_frame;
    if drain {
        max_out_samples += (*(f as *mut PrivData)).bytes_queued;
    }
    out = mp_aframe_new_ref((*(f as *mut PrivData)).cur_format);
    if mp_aframe_pool_allocate((*(f as *mut PrivData)).out_pool, out, max_out_samples) < 0 {
        goto error;
    }
    if !out.is_null() {
        mp_aframe_copy_attributes(out, (*(f as *mut PrivData)).in_);
    }
    let mut out_planes = mp_aframe_get_data_rw(out);
    if out_planes.is_null() {
        goto error;
    }
    let mut pout = out_planes[0] as *mut i8;
    let mut out_offset = 0;
    if (*(f as *mut PrivData)).bytes_queued >= (*(f as *mut PrivData)).bytes_queue {
        let mut ti: i32;
        let mut tf: f32;
        let mut bytes_off = 0;
        if (*(f as *mut PrivData)).output_overlap.is_some() {
            if (*(f as *mut PrivData)).best_overlap_offset.is_some() {
                bytes_off = (*(f as *mut PrivData)).best_overlap_offset.unwrap()(f);
            }
            (*(f as *mut PrivData)).output_overlap.unwrap()(f, pout.offset(out_offset as isize), bytes_off);
        }
        memcpy(pout.offset(out_offset as isize).offset((*(f as *mut PrivData)).bytes_overlap as isize) as *mut _, (*(f as *mut PrivData)).buf_queue.offset(bytes_off as isize).offset((*(f as *mut PrivData)).bytes_overlap as isize) as *mut _, (*(f as *mut PrivData)).bytes_standing as usize);
        out_offset += (*(f as *mut PrivData)).bytes_stride;
        memcpy((*(f as *mut PrivData)).buf_overlap as *mut _, (*(f as *mut PrivData)).buf_queue.offset(bytes_off as isize).offset((*(f as *mut PrivData)).bytes_stride as isize) as *mut _, (*(f as *mut PrivData)).bytes_overlap as usize);
        tf = (*(f as *mut PrivData)).frames_stride_scaled + (*(f as *mut PrivData)).frames_stride_error;
        ti = tf as i32;
        (*(f as *mut PrivData)).frames_stride_error = tf - ti as f32;
        (*(f as *mut PrivData)).bytes_to_slide = ti * (*(f as *mut PrivData)).bytes_per_frame;
    }
    if drain && (*(f as *mut PrivData)).bytes_queued > 0 {
        memcpy(pout.offset(out_offset as isize), (*(f as *mut PrivData)).buf_queue as *mut _, (*(f as *mut PrivData)).bytes_queued as usize);
        out_offset += (*(f as *mut PrivData)).bytes_queued;
        (*(f as *mut PrivData)).bytes_queued = 0;
    }
    mp_aframe_set_size(out, out_offset / (*(f as *mut PrivData)).bytes_per_frame);
    let mut delay = (out_offset as f32 * (*(f as *mut PrivData)).speed + (*(f as *mut PrivData)).bytes_queued as f32 - (*(f as *mut PrivData)).bytes_to_slide as f32) / (*(f as *mut PrivData)).bytes_per_frame as f32 / mp_aframe_get_effective_rate(out) as f32 + if !(*(f as *mut PrivData)).in_.is_null() { mp_aframe_duration((*(f as *mut PrivData)).in_) } else { 0.0 };
    if (*(f as *mut PrivData)).current_pts != MP_NOPTS_VALUE {
        mp_aframe_set_pts(out, (*(f as *mut PrivData)).current_pts - delay);
    }
    mp_aframe_mul_speed(out, (*(f as *mut PrivData)).speed);
    if mp_aframe_get_size(out) == 0 {
        TA_FREEP(&mut out);
    }
    if is_eof && !out.is_null() {
        mp_pin_out_repeat_eof((*(f as *mut PrivData)).in_pin);
    } else if is_eof && out.is_null() {
        mp_pin_in_write((*(f as *mut PrivData)).ppins[1], MP_EOF_FRAME);
    } else if !is_eof && out.is_null() {
        mp_pin_out_request_data_next((*(f as *mut PrivData)).in_pin);
    }
    if !out.is_null() {
        mp_pin_in_write((*(f as *mut PrivData)).ppins[1], MAKE_FRAME(MP_FRAME_AUDIO, out));
    }
    return;
error:
    TA_FREEP(&mut (*(f as *mut PrivData)).in_);
    talloc_free(out);
    mp_filter_internal_mark_failed(f);
}

fn update_speed(f: *mut mp_filter, speed: f32) {
    (*(f as *mut PrivData)).speed = speed;
    let factor = if (*(f as *mut PrivData)).opts.speed_opt & SCALE_PITCH != 0 { 1.0 / (*(f as *mut PrivData)).speed } else { (*(f as *mut PrivData)).speed };
    (*(f as *mut PrivData)).scale = factor * (*(f as *mut PrivData)).opts.scale_nominal;
    (*(f as *mut PrivData)).frames_stride_scaled = (*(f as *mut PrivData)).scale * (*(f as *mut PrivData)).frames_stride as f32;
    (*(f as *mut PrivData)).frames_stride_error = min((*(f as *mut PrivData)).frames_stride_error, (*(f as *mut PrivData)).frames_stride_scaled);
}

fn command(f: *mut mp_filter, cmd: *mut mp_filter_command) -> bool {
    let s = (*(f as *mut PrivData)).opts;
    if (*cmd).type_ == MP_FILTER_COMMAND_SET_SPEED {
        if s.speed_opt & SCALE_TEMPO != 0 {
            if s.speed_opt & SCALE_PITCH != 0 {
                return false;
            }
            update_speed(f, (*cmd).speed);
            return true;
        } else if s.speed_opt & SCALE_PITCH != 0 {
            update_speed(f, (*cmd).speed);
            return false;
        }
    }
    false
}

fn reset(f: *mut mp_filter) {
    let s = (*(f as *mut PrivData)).opts;
    (*(f as *mut PrivData)).current_pts = MP_NOPTS_VALUE;
    (*(f as *mut PrivData)).bytes_queued = 0;
    (*(f as *mut PrivData)).bytes_to_slide = 0;
    (*(f as *mut PrivData)).frames_stride_error = 0.0;
    if !(*(f as *mut PrivData)).buf_overlap.is_null() && (*(f as *mut PrivData)).bytes_overlap > 0 {
        memset((*(f as *mut PrivData)).buf_overlap as *mut _, 0, (*(f as *mut PrivData)).bytes_overlap);
    }
    TA_FREEP(&mut (*(f as *mut PrivData)).in_);
}

fn destroy(f: *mut mp_filter) {
    let s = (*(f as *mut PrivData)).opts;
    free((*(f as *mut PrivData)).buf_queue as *mut _);
    free((*(f as *mut PrivData)).buf_overlap as *mut _);
    free((*(f as *mut PrivData)).buf_pre_corr as *mut _);
    free((*(f as *mut PrivData)).table_blend as *mut _);
    free((*(f as *mut PrivData)).table_window as *mut _);
    TA_FREEP(&mut (*(f as *mut PrivData)).in_);
    mp_filter_free_children(f);
}

const af_scaletempo_filter: mp_filter_info = mp_filter_info {
    name: "scaletempo",
    priv_size: mem::size_of::<PrivData>(),
    process: process,
    command: command,
    reset: reset,
    destroy: destroy,
};

fn af_scaletempo_create(parent: *mut mp_filter, options: *mut c_void) -> *mut mp_filter {
    let f = mp_filter_create(parent, &af_scaletempo_filter);
    if f.is_null() {
        talloc_free(options);
        return ptr::null_mut();
    }
    mp_filter_add_pin(f, MP_PIN_IN, "in");
    mp_filter_add_pin(f, MP_PIN_OUT, "out");
    let s = (*(f as *mut PrivData)).opts;
    (*(f as *mut PrivData)).opts = unsafe { *(options as *mut FOpts) };
    (*(f as *mut PrivData)).speed = 1.0;
    (*(f as *mut PrivData)).cur_format = mp_aframe_create();
    (*(f as *mut PrivData)).out_pool = mp_aframe_pool_create(f);
    let conv = mp_autoconvert_create(f);
    if conv.is_null() {
        abort();
    }
    mp_autoconvert_add_afmt(conv, AF_FORMAT_S16);
    mp_autoconvert_add_afmt(conv, AF_FORMAT_FLOAT);
    mp_pin_connect(conv.f.pins[0], (*(f as *mut PrivData)).ppins[0]);
    (*(f as *mut PrivData)).in_pin = conv.f.pins[1];
    f
}

const af_scaletempo: mp_user_filter_entry = mp_user_filter_entry {
    desc: mp_user_filter_desc {
        description: "Scale audio tempo while maintaining pitch",
        name: "scaletempo",
        priv_size: mem::size_of::<FOpts>(),
        priv_defaults: &FOpts {
            scale_nominal: 1.0,
            ms_stride: 60.0,
            factor_overlap: 0.20,
            ms_search: 14.0,
            speed_opt: SCALE_TEMPO,
        },
        options: &[
            mp_option {
                name: b"scale\0" as *const u8 as *const c_char,
                type_: M_OPT_TYPE_FLOAT,
                dest: mp_offsetof!(f_opts, scale_nominal),
                min: 0.01,
                max: f32::MAX,
            },
            mp_option {
                name: b"stride\0" as *const u8 as *const c_char,
                type_: M_OPT_TYPE_FLOAT,
                dest: mp_offsetof!(f_opts, ms_stride),
                min: 0.01,
                max: f32::MAX,
            },
            mp_option {
                name: b"overlap\0" as *const u8 as *const c_char,
                type_: M_OPT_TYPE_FLOAT,
                dest: mp_offsetof!(f_opts, factor_overlap),
                min: 0.0,
                max: 1.0,
            },
            mp_option {
                name: b"search\0" as *const u8 as *const c_char,
                type_: M_OPT_TYPE_FLOAT,
                dest: mp_offsetof!(f_opts, ms_search),
                min: 0.0,
                max: f32::MAX,
            },
            mp_option {
                name: b"speed\0" as *const u8 as *const c_char,
                type_: M_OPT_TYPE_CHOICE,
                dest: mp_offsetof!(f_opts, speed_opt),
                min: 0,
                max: 3,
                choices: [
                    mp_choice {
                        name: b"pitch\0" as *const u8 as *const c_char,
                        value: SCALE_PITCH,
                    },
                    mp_choice {
                        name: b"tempo\0" as *const u8 as *const c_char,
                        value: SCALE_TEMPO,
                    },
                    mp_choice {
                        name: b"none\0" as *const u8 as *const c_char,
                        value: 0,
                    },
                    mp_choice {
                        name: b"both\0" as *const u8 as *const c_char,
                        value: SCALE_TEMPO | SCALE_PITCH,
                    },
                    mp_choice {
                        name: ptr::null(),
                        value: 0,
                    },
                ].as_ptr(),
            },
            mp_option {
                name: ptr::null(),
                type_: M_OPT_TYPE_UNKNOWN,
                dest: 0,
                min: 0.0,
                max: 0.0,
            },
        ].as_ptr(),
    },
    create: af_scaletempo_create,
};


