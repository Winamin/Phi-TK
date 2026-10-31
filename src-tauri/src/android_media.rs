use anyhow::{bail, Context, Result};
use std::{
    ffi::CString,
    fs::File,
    os::{fd::AsRawFd, raw::c_void},
    ptr,
};

const COLOR_FORMAT_SURFACE: i32 = 0x7F000789;
const COLOR_FORMAT_SEMI_PLANAR: i32 = 21;

const BUFFER_FLAG_CODEC_CONFIG: u32 = 2;
const BUFFER_FLAG_PARTIAL_FRAME: u32 = 8;
const BUFFER_FLAG_END_OF_STREAM: u32 = 4;

const INFO_TRY_AGAIN_LATER: isize = -1;
const INFO_OUTPUT_FORMAT_CHANGED: isize = -2;
const INFO_OUTPUT_BUFFERS_CHANGED: isize = -3;

const CONFIGURE_FLAG_ENCODE: u32 = 1;

const MUXER_OUTPUT_MPEG_4: i32 = 0;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct BufferInfo {
    pub offset: i32,
    pub size: i32,
    pub presentation_time_us: i64,
    pub flags: u32,
}

#[repr(C)]
pub struct AMediaFormat {
    _private: [u8; 0],
}

#[repr(C)]
pub struct AMediaCodec {
    _private: [u8; 0],
}

#[repr(C)]
pub struct AMediaMuxer {
    _private: [u8; 0],
}

#[link(name = "mediandk")]
extern "C" {
    fn AMediaFormat_new() -> *mut AMediaFormat;
    fn AMediaFormat_delete(format: *mut AMediaFormat) -> i32;
    fn AMediaFormat_setInt32(format: *mut AMediaFormat, name: *const std::os::raw::c_char, value: i32);
    fn AMediaFormat_setInt64(format: *mut AMediaFormat, name: *const std::os::raw::c_char, value: i64);
    fn AMediaFormat_setFloat(format: *mut AMediaFormat, name: *const std::os::raw::c_char, value: f32);
    fn AMediaFormat_setString(
        format: *mut AMediaFormat,
        name: *const std::os::raw::c_char,
        value: *const std::os::raw::c_char,
    );
    fn AMediaFormat_getInt32(
        format: *mut AMediaFormat,
        name: *const std::os::raw::c_char,
        out: *mut i32,
    ) -> bool;
    fn AMediaFormat_getBuffer(
        format: *mut AMediaFormat,
        name: *const std::os::raw::c_char,
        data: *mut *mut c_void,
        size: *mut usize,
    ) -> bool;
    fn AMediaFormat_toString(format: *mut AMediaFormat) -> *mut std::os::raw::c_char;
    fn AMediaFormat_setBuffer(
        format: *mut AMediaFormat,
        name: *const std::os::raw::c_char,
        data: *const c_void,
        size: usize,
    );

    fn AMediaCodec_createEncoderByType(mime: *const std::os::raw::c_char) -> *mut AMediaCodec;
    fn AMediaCodec_createCodecByName(name: *const std::os::raw::c_char) -> *mut AMediaCodec;
    fn AMediaCodec_configure(
        codec: *mut AMediaCodec,
        format: *const AMediaFormat,
        surface: *mut c_void,
        crypto: *mut c_void,
        flags: u32,
    ) -> i32;
    fn AMediaCodec_start(codec: *mut AMediaCodec) -> i32;
    fn AMediaCodec_stop(codec: *mut AMediaCodec) -> i32;
    fn AMediaCodec_flush(codec: *mut AMediaCodec) -> i32;
    fn AMediaCodec_delete(codec: *mut AMediaCodec) -> i32;
    fn AMediaCodec_getInputBuffer(codec: *mut AMediaCodec, idx: usize, out_size: *mut usize) -> *mut u8;
    fn AMediaCodec_getOutputBuffer(codec: *mut AMediaCodec, idx: usize, out_size: *mut usize) -> *mut u8;
    fn AMediaCodec_dequeueInputBuffer(codec: *mut AMediaCodec, timeout_us: i64) -> isize;
    fn AMediaCodec_queueInputBuffer(
        codec: *mut AMediaCodec,
        idx: usize,
        offset: usize,
        size: usize,
        time: i64,
        flags: u32,
    ) -> i32;
    fn AMediaCodec_dequeueOutputBuffer(
        codec: *mut AMediaCodec,
        info: *mut BufferInfo,
        timeout_us: i64,
    ) -> isize;
    fn AMediaCodec_getOutputFormat(codec: *mut AMediaCodec) -> *mut AMediaFormat;
    fn AMediaCodec_releaseOutputBuffer(codec: *mut AMediaCodec, idx: usize, render: bool) -> i32;

    fn AMediaMuxer_new(fd: i32, format: i32) -> *mut AMediaMuxer;
    fn AMediaMuxer_delete(muxer: *mut AMediaMuxer) -> i32;
    fn AMediaMuxer_addTrack(muxer: *mut AMediaMuxer, format: *const AMediaFormat) -> isize;
    fn AMediaMuxer_start(muxer: *mut AMediaMuxer) -> i32;
    fn AMediaMuxer_stop(muxer: *mut AMediaMuxer) -> i32;
    fn AMediaMuxer_writeSampleData(
        muxer: *mut AMediaMuxer,
        track: usize,
        data: *const u8,
        info: *const BufferInfo,
    ) -> i32;
}

fn codec_name(codec: *mut AMediaCodec) -> Option<String> {
    unsafe {
        let symbol = libc_dlsym(b"AMediaCodec_getName\0".as_ptr() as *const std::os::raw::c_char);
        if symbol.is_null() {
            return None;
        }
        let get_name: extern "C" fn(*mut AMediaCodec, *mut *mut std::os::raw::c_char) -> i32 =
            std::mem::transmute(symbol);
        let mut out: *mut std::os::raw::c_char = ptr::null_mut();
        if get_name(codec, &mut out) != 0 || out.is_null() {
            return None;
        }
        let name = std::ffi::CStr::from_ptr(out).to_string_lossy().into_owned();
        libc_free(out as *mut c_void);
        Some(name)
    }
}

unsafe fn libc_dlsym(name: *const std::os::raw::c_char) -> *mut c_void {
    extern "C" {
        fn dlsym(handle: *mut c_void, symbol: *const std::os::raw::c_char) -> *mut c_void;
        fn dlopen(filename: *const std::os::raw::c_char, flag: i32) -> *mut c_void;
    }
    const RTLD_NOW: i32 = 2;
    let handle = dlopen(b"libmediandk.so\0".as_ptr() as *const std::os::raw::c_char, RTLD_NOW);
    if handle.is_null() {
        return ptr::null_mut();
    }
    dlsym(handle, name)
}

unsafe fn libc_free(pointer: *mut c_void) {
    extern "C" {
        fn free(pointer: *mut c_void);
    }
    free(pointer);
}

fn is_hardware(name: &str) -> bool {
    name.starts_with("c2.qti") || name.starts_with("OMX.qcom") || name.contains(".qti.")
}

fn video_codec_for(codec: &str) -> (&'static str, &'static [&'static str], &'static str) {
    if codec.eq_ignore_ascii_case("hevc") || codec.eq_ignore_ascii_case("h265") {
        (
            "video/hevc",
            &["c2.qti.hevc.encoder", "OMX.qcom.video.encoder.hevc"],
            "HEVC",
        )
    } else {
        (
            "video/avc",
            &["c2.qti.avc.encoder", "OMX.qcom.video.encoder.avc"],
            "H.264",
        )
    }
}

fn create_hardware_video_encoder(
    mime: &CString,
    candidates: &[&str],
) -> Option<*mut AMediaCodec> {
    let codec = unsafe { AMediaCodec_createEncoderByType(mime.as_ptr()) };
    if !codec.is_null() {
        if let Some(name) = codec_name(codec) {
            if is_hardware(&name) {
                crate::android_uri::debug_log(&format!("Selected encoder: {name}"));
                return Some(codec);
            }
            crate::android_uri::debug_log(&format!(
                "encoder: default codec {name} is not hardware, looking for a Qualcomm one"
            ));
        } else {
            return Some(codec);
        }
        unsafe {
            AMediaCodec_delete(codec);
        }
    }

    for candidate in candidates {
        let Ok(name) = CString::new(*candidate) else {
            continue;
        };
        let codec = unsafe { AMediaCodec_createCodecByName(name.as_ptr()) };
        if codec.is_null() {
            continue;
        }
        match codec_name(codec) {
            Some(actual) if is_hardware(&actual) => {
                crate::android_uri::debug_log(&format!("Selected encoder: {actual}"));
                return Some(codec);
            }
            _ => unsafe {
                AMediaCodec_delete(codec);
            },
        }
    }

    let fallback = unsafe { AMediaCodec_createEncoderByType(mime.as_ptr()) };
    if !fallback.is_null() {
        if let Some(name) = codec_name(fallback) {
            crate::android_uri::debug_log(&format!(
                "encoder: falling back to {name} (no hardware encoder found)"
            ));
        }
        return Some(fallback);
    }
    None
}

fn cstr(value: &str) -> CString {
    CString::new(value).unwrap()
}

pub struct TrackFormat {
    pub(crate) format: *mut AMediaFormat,
    csd: Vec<Vec<u8>>,
}

impl TrackFormat {
    fn from_codec(codec: *mut AMediaCodec) -> Result<Self> {
        let format = unsafe { AMediaCodec_getOutputFormat(codec) };
        if format.is_null() {
            bail!("codec did not report an output format");
        }
        let mut csd = Vec::new();
        for name in ["csd-0", "csd-1", "csd-2"] {
            let key = cstr(name);
            let mut data: *mut c_void = ptr::null_mut();
            let mut size: usize = 0;
            let ok = unsafe { AMediaFormat_getBuffer(format, key.as_ptr(), &mut data, &mut size) };
            if ok && !data.is_null() && size > 0 {
                let bytes = unsafe { std::slice::from_raw_parts(data as *const u8, size) };
                csd.push(bytes.to_vec());
            }
        }
        Ok(Self { format, csd })
    }

    pub fn width(&self) -> Option<i32> {
        let key = cstr("width");
        let mut value = 0;
        let ok = unsafe { AMediaFormat_getInt32(self.format, key.as_ptr(), &mut value) };
        ok.then_some(value)
    }
}

impl Drop for TrackFormat {
    fn drop(&mut self) {
        if !self.format.is_null() {
            unsafe {
                AMediaFormat_delete(self.format);
            }
        }
    }
}

pub struct Encoder {
    codec: *mut AMediaCodec,
    width: u32,
    height: u32,
    strided: bool,
    pending: Vec<(Vec<u8>, i64, u32)>,
    format: Option<TrackFormat>,
    first_config: Option<Vec<u8>>,
    first_buffer_size: Option<usize>,
    frames_in: u64,
}

pub struct EncodedPacket {
    pub data: Vec<u8>,
    pub time_us: i64,
    pub keyframe: bool,
    pub config: bool,
}

impl Encoder {
    pub fn video(
        width: u32,
        height: u32,
        bitrate: u32,
        fps: u32,
        keyframe_interval_seconds: u32,
        codec_kind: &str,
    ) -> Result<Self> {
        let (mime_name, candidates, label) = video_codec_for(codec_kind);
        let mime = cstr(mime_name);
        let codec =
            create_hardware_video_encoder(&mime, candidates).unwrap_or(ptr::null_mut());
        if codec.is_null() {
            bail!("no {label} encoder is available on this device")
        }
        let format = unsafe { AMediaFormat_new() };
        if format.is_null() {
            unsafe {
                AMediaCodec_delete(codec);
            }
            bail!("failed to allocate the encoder media format")
        }
        let key_mime = cstr("mime");
        let key_width = cstr("width");
        let key_height = cstr("height");
        let key_bitrate = cstr("bitrate");
        let key_fps = cstr("frame-rate");
        let key_i_frame = cstr("i-frame-interval");
        let key_color = cstr("color-format");
        let key_profile = cstr("profile");
        let key_level = cstr("level");
        let key_bitrate_mode = cstr("bitrate-mode");
        unsafe {
            AMediaFormat_setString(format, key_mime.as_ptr(), mime.as_ptr());
            AMediaFormat_setInt32(format, key_width.as_ptr(), width as i32);
            AMediaFormat_setInt32(format, key_height.as_ptr(), height as i32);
            AMediaFormat_setInt32(format, key_bitrate.as_ptr(), bitrate as i32);
            AMediaFormat_setInt32(format, key_fps.as_ptr(), fps.max(1) as i32);
            AMediaFormat_setInt32(format, key_i_frame.as_ptr(), keyframe_interval_seconds.max(1) as i32);
            AMediaFormat_setInt32(format, key_color.as_ptr(), COLOR_FORMAT_SEMI_PLANAR);
            AMediaFormat_setInt32(format, key_profile.as_ptr(), 1);
            AMediaFormat_setInt32(format, key_level.as_ptr(), 0x2000);
            AMediaFormat_setInt32(format, key_bitrate_mode.as_ptr(), 2);
        }
        let status = unsafe {
            AMediaCodec_configure(codec, format, ptr::null_mut(), ptr::null_mut(), CONFIGURE_FLAG_ENCODE)
        };
        unsafe {
            AMediaFormat_delete(format);
        }
        if status < 0 {
            unsafe {
                AMediaCodec_delete(codec);
            }
            bail!("failed to configure the {label} encoder (status {status})")
        }
        let status = unsafe { AMediaCodec_start(codec) };
        if status < 0 {
            unsafe {
                AMediaCodec_delete(codec);
            }
            bail!("failed to start the {label} encoder (status {status})")
        }
        crate::android_uri::debug_log(&format!(
            "video encoder: {label} {}x{} {}bps @ {fps}fps keyframe={}",
            width, height, bitrate, keyframe_interval_seconds
        ));
        Ok(Self {
            codec,
            width,
            height,
            strided: true,
            pending: Vec::new(),
            format: None,
            first_config: None,
            first_buffer_size: None,
            frames_in: 0,
        })
    }

    pub fn audio(sample_rate: u32, channels: u32, bitrate: u32) -> Result<Self> {
        let mime = cstr("audio/mp4a-latm");
        let codec = unsafe { AMediaCodec_createEncoderByType(mime.as_ptr()) };
        if codec.is_null() {
            bail!("no AAC encoder is available on this device")
        }
        let format = unsafe { AMediaFormat_new() };
        if format.is_null() {
            unsafe {
                AMediaCodec_delete(codec);
            }
            bail!("failed to allocate the encoder media format")
        }
        let key_mime = cstr("mime");
        let key_rate = cstr("sample-rate");
        let key_channels = cstr("channel-count");
        let key_bitrate = cstr("bitrate");
        let key_profile = cstr("aac-profile");
        let key_format = cstr("color-format");
        unsafe {
            AMediaFormat_setString(format, key_mime.as_ptr(), mime.as_ptr());
            AMediaFormat_setInt32(format, key_rate.as_ptr(), sample_rate as i32);
            AMediaFormat_setInt32(format, key_channels.as_ptr(), channels as i32);
            AMediaFormat_setInt32(format, key_bitrate.as_ptr(), bitrate as i32);
            AMediaFormat_setInt32(format, key_profile.as_ptr(), 2);
            AMediaFormat_setInt32(format, key_format.as_ptr(), 1);
        }
        let status = unsafe {
            AMediaCodec_configure(codec, format, ptr::null_mut(), ptr::null_mut(), CONFIGURE_FLAG_ENCODE)
        };
        unsafe {
            AMediaFormat_delete(format);
        }
        if status < 0 {
            unsafe {
                AMediaCodec_delete(codec);
            }
            bail!("failed to configure the AAC encoder (status {status})")
        }
        let status = unsafe { AMediaCodec_start(codec) };
        if status < 0 {
            unsafe {
                AMediaCodec_delete(codec);
            }
            bail!("failed to start the AAC encoder (status {status})")
        }
        Ok(Self {
            codec,
            width: channels,
            height: 16,
            strided: false,
            pending: Vec::new(),
            format: None,
            first_config: None,
            first_buffer_size: None,
            frames_in: 0,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn queued(&self) -> usize {
        self.pending.len()
    }

    pub fn take_format(&mut self) -> Option<TrackFormat> {
        self.format.take()
    }

    pub fn first_config(&self) -> Option<Vec<u8>> {
        self.first_config.clone()
    }

    pub fn describe_format(format: &TrackFormat) -> String {
        let mut width = 0;
        let mut height = 0;
        let mut color = 0;
        let key_width = cstr("width");
        let key_height = cstr("height");
        let key_color = cstr("color-format");
        unsafe {
            AMediaFormat_getInt32(format.format, key_width.as_ptr(), &mut width);
            AMediaFormat_getInt32(format.format, key_height.as_ptr(), &mut height);
            AMediaFormat_getInt32(format.format, key_color.as_ptr(), &mut color);
        }
        let text = unsafe { AMediaFormat_toString(format.format) };
        let detail = if text.is_null() {
            String::from("<null>")
        } else {
            unsafe { std::ffi::CStr::from_ptr(text) }
                .to_string_lossy()
                .into_owned()
        };
        format!("{width}x{height} color-format={color} keys={detail}")
    }

    pub fn submit(&mut self, data: &[u8], time_us: i64, keyframe: bool) -> Result<()> {
        if self.strided {
            let expected = (self.width * self.height * 3 / 2) as usize;
            if data.len() != expected {
                crate::android_uri::debug_log(&format!(
                    "encoder: frame is {} bytes but {}x{} NV12 needs {expected}",
                    data.len(),
                    self.width,
                    self.height
                ));
                bail!(
                    "frame payload is {} bytes, expected {expected} for {}x{} NV12",
                    data.len(),
                    self.width,
                    self.height
                );
            }
        }
        self.pending.push((data.to_vec(), time_us, if keyframe { 1 } else { 0 }));
        self.frames_in += 1;
        self.pump()
    }

    fn pump(&mut self) -> Result<()> {
        while let Some((data, time_us, flags)) = self.pending.first().cloned() {
            let mut written = 0usize;
            while written < data.len() {
                let idx = unsafe { AMediaCodec_dequeueInputBuffer(self.codec, 8_000) };
                if idx == INFO_TRY_AGAIN_LATER {
                    break;
                }
                if idx < 0 {
                    bail!("failed to dequeue an encoder input buffer (status {idx})")
                }
                let idx = idx as usize;
                let mut size = 0usize;
                let buffer = unsafe { AMediaCodec_getInputBuffer(self.codec, idx, &mut size) };
                if buffer.is_null() {
                    bail!("encoder returned a null input buffer")
                }
                if size == 0 {
                    bail!("encoder returned an empty input buffer")
                }
                if self.first_buffer_size.is_none() {
                    self.first_buffer_size = Some(size);
                    crate::android_uri::debug_log(&format!(
                        "encoder: input buffer size {size}, frame size {}",
                        data.len()
                    ));
                }
                let chunk = size.min(data.len() - written);
                unsafe {
                    ptr::copy_nonoverlapping(data.as_ptr().add(written), buffer, chunk);
                }
                let partial = if written + chunk < data.len() {
                    BUFFER_FLAG_PARTIAL_FRAME
                } else {
                    0
                };
                let stamp = if written == 0 { time_us } else { 0 };
                let status = unsafe {
                    AMediaCodec_queueInputBuffer(
                        self.codec,
                        idx,
                        0,
                        chunk,
                        stamp,
                        flags | partial,
                    )
                };
                if status < 0 {
                    bail!("failed to queue an encoder input buffer (status {status})")
                }
                written += chunk;
            }
            if written < data.len() {
                break;
            }
            self.pending.remove(0);
        }
        Ok(())
    }

    pub fn drain(&mut self) -> Result<Vec<EncodedPacket>> {
        let mut out = Vec::new();
        loop {
            let mut info = BufferInfo::default();
            let idx = unsafe { AMediaCodec_dequeueOutputBuffer(self.codec, &mut info, 0) };
            if idx == INFO_TRY_AGAIN_LATER {
                break;
            }
            if idx == INFO_OUTPUT_BUFFERS_CHANGED {
                continue;
            }
            if idx == INFO_OUTPUT_FORMAT_CHANGED {
                self.format = Some(TrackFormat::from_codec(self.codec)?);
                continue;
            }
            if idx < 0 {
                bail!("failed to dequeue an encoder output buffer (status {idx})")
            }
            let idx = idx as usize;
            let mut size = 0usize;
            let buffer = unsafe { AMediaCodec_getOutputBuffer(self.codec, idx, &mut size) };
            if !buffer.is_null() && info.size > 0 {
                let offset = info.offset as usize;
                let len = info.size as usize;
                if offset + len <= size {
                    let data = unsafe {
                        std::slice::from_raw_parts(buffer.add(offset), len).to_vec()
                    };
                    let config = info.flags & BUFFER_FLAG_CODEC_CONFIG != 0;
                    if config {
                        if self.first_config.is_none() {
                            self.first_config = Some(data);
                        }
                    } else {
                        out.push(EncodedPacket {
                            data,
                            time_us: info.presentation_time_us,
                            keyframe: info.flags & 1 != 0,
                            config: false,
                        });
                    }
                }
            }
            unsafe {
                AMediaCodec_releaseOutputBuffer(self.codec, idx, false);
            }
            if info.flags & BUFFER_FLAG_END_OF_STREAM != 0 {
                break;
            }
        }
        Ok(out)
    }

    pub fn flush(&mut self) -> Result<Vec<EncodedPacket>> {
        let mut out = Vec::new();
        let mut idle = 0;
        loop {
            let idx = unsafe { AMediaCodec_dequeueInputBuffer(self.codec, 8_000) };
            if idx >= 0 {
                let idx = idx as usize;
                let status = unsafe {
                    AMediaCodec_queueInputBuffer(self.codec, idx, 0, 0, 0, BUFFER_FLAG_END_OF_STREAM)
                };
                if status < 0 {
                    bail!("failed to signal end of stream to the encoder (status {status})")
                }
                break;
            }
            idle += 1;
            if idle > 200 {
                break;
            }
        }
        loop {
            let mut info = BufferInfo::default();
            let idx = unsafe { AMediaCodec_dequeueOutputBuffer(self.codec, &mut info, 10_000) };
            if idx == INFO_TRY_AGAIN_LATER {
                if idle > 200 {
                    break;
                }
                idle += 1;
                continue;
            }
            if idx == INFO_OUTPUT_BUFFERS_CHANGED {
                continue;
            }
            if idx == INFO_OUTPUT_FORMAT_CHANGED {
                self.format = Some(TrackFormat::from_codec(self.codec)?);
                continue;
            }
            if idx < 0 {
                bail!("failed to drain the encoder tail (status {idx})")
            }
            let idx = idx as usize;
            let mut size = 0usize;
            let buffer = unsafe { AMediaCodec_getOutputBuffer(self.codec, idx, &mut size) };
            let mut eos = false;
            if !buffer.is_null() && info.size > 0 {
                let offset = info.offset as usize;
                let len = info.size as usize;
                if offset + len <= size {
                    let data =
                        unsafe { std::slice::from_raw_parts(buffer.add(offset), len).to_vec() };
                    if info.flags & BUFFER_FLAG_CODEC_CONFIG != 0 {
                        if self.first_config.is_none() {
                            self.first_config = Some(data);
                        }
                    } else {
                        out.push(EncodedPacket {
                            data,
                            time_us: info.presentation_time_us,
                            keyframe: info.flags & 1 != 0,
                            config: false,
                        });
                    }
                }
            }
            eos = info.flags & BUFFER_FLAG_END_OF_STREAM != 0;
            unsafe {
                AMediaCodec_releaseOutputBuffer(self.codec, idx, false);
            }
            if eos {
                break;
            }
        }
        Ok(out)
    }

    pub fn release(&mut self) {
        if self.codec.is_null() {
            return;
        }
        unsafe {
            AMediaCodec_stop(self.codec);
            AMediaCodec_delete(self.codec);
        }
        self.codec = ptr::null_mut();
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        self.release();
    }
}

pub struct Muxer {
    file: File,
    muxer: *mut AMediaMuxer,
    video_track: Option<usize>,
    audio_track: Option<usize>,
    video_format: Option<TrackFormat>,
    audio_format: Option<TrackFormat>,
    started: bool,
    stopped: bool,
    video_samples: u64,
    audio_samples: u64,
    unknown_samples: u64,
}

impl Muxer {
    pub fn new(path: &std::path::Path) -> Result<Self> {
        let file = File::create(path)
            .with_context(|| format!("failed to create {}", path.display()))?;
        let muxer = unsafe { AMediaMuxer_new(file.as_raw_fd(), MUXER_OUTPUT_MPEG_4) };
        if muxer.is_null() {
            bail!("failed to create the MP4 muxer for {}", path.display())
        }
        Ok(Self {
            file,
            muxer,
            video_track: None,
            audio_track: None,
            video_format: None,
            audio_format: None,
            started: false,
            stopped: false,
            video_samples: 0,
            audio_samples: 0,
            unknown_samples: 0,
        })
    }

    pub fn set_video_track(&mut self, format: TrackFormat) -> Result<usize> {
        let index = unsafe { AMediaMuxer_addTrack(self.muxer, format.format) };
        if index < 0 {
            bail!("failed to add the video track to the muxer (status {index})")
        }
        let index = index as usize;
        self.video_track = Some(index);
        self.video_format = Some(format);
        Ok(index)
    }

    pub fn set_audio_track(&mut self, format: TrackFormat) -> Result<usize> {
        let index = unsafe { AMediaMuxer_addTrack(self.muxer, format.format) };
        if index < 0 {
            bail!("failed to add the audio track to the muxer (status {index})")
        }
        let index = index as usize;
        self.audio_track = Some(index);
        self.audio_format = Some(format);
        Ok(index)
    }

    pub fn start(&mut self) -> Result<()> {
        if self.started {
            return Ok(());
        }
        if self.video_track.is_none() && self.audio_track.is_none() {
            bail!("cannot start the muxer without any track")
        }
        let status = unsafe { AMediaMuxer_start(self.muxer) };
        if status < 0 {
            bail!("failed to start the MP4 muxer (status {status})")
        }
        self.started = true;
        Ok(())
    }

    pub fn video_track(&self) -> Option<usize> {
        self.video_track
    }

    pub fn audio_track(&self) -> Option<usize> {
        self.audio_track
    }

    pub fn write(&mut self, track: usize, data: &[u8], time_us: i64) -> Result<()> {
        self.write_with_flags(track, data, time_us, false)
    }

    pub fn write_with_flags(
        &mut self,
        track: usize,
        data: &[u8],
        time_us: i64,
        keyframe: bool,
    ) -> Result<()> {
        if !self.started || self.stopped {
            crate::android_uri::debug_log(&format!(
                "muxer: dropping sample for track {track} (started={} stopped={})",
                self.started, self.stopped
            ));
            return Ok(());
        }
        if Some(track) == self.video_track {
            self.video_samples += 1;
        } else if Some(track) == self.audio_track {
            self.audio_samples += 1;
        } else {
            self.unknown_samples += 1;
        }
        let info = BufferInfo {
            offset: 0,
            size: data.len() as i32,
            presentation_time_us: time_us,
            flags: if keyframe { 1 } else { 0 },
        };
        let status = unsafe {
            AMediaMuxer_writeSampleData(self.muxer, track, data.as_ptr(), &info)
        };
        if status < 0 {
            bail!("failed to write a sample into the MP4 muxer (status {status})")
        }
        Ok(())
    }

    pub fn finish(&mut self) -> Result<()> {
        crate::android_uri::debug_log(&format!(
            "muxer: finish video={} audio={} unknown={} tracks video={:?} audio={:?} started={}",
            self.video_samples,
            self.audio_samples,
            self.unknown_samples,
            self.video_track,
            self.audio_track,
            self.started
        ));
        if self.stopped {
            return Ok(());
        }
        self.stopped = true;
        if !self.muxer.is_null() {
            if self.started {
                unsafe {
                    AMediaMuxer_stop(self.muxer);
                }
            }
            unsafe {
                AMediaMuxer_delete(self.muxer);
            }
            self.muxer = ptr::null_mut();
        }
        let _ = self.file.sync_all();
        Ok(())
    }
}

impl Drop for Muxer {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

pub fn pcm_f32_to_i16(samples: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(samples.len() * 2);
    for sample in samples {
        let value = (sample.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        out.extend_from_slice(&value.to_le_bytes());
    }
    out
}
