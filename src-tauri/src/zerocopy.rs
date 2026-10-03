#![allow(dead_code, unused_unsafe, unused_mut)]

use crate::render::{cmd_hidden, new_plane};
use crate::Path;
use anyhow::{Context, Result};
use macroquad::{miniquad::{TextureFormat}, prelude::*};
use prpr::core::internal_id;
use std::process::Stdio;

#[cfg(target_os = "windows")]
pub mod nvenc {
    use super::*;
    use std::ffi::c_void;
    use windows::core::Interface;
    use windows::Win32::Graphics::Direct3D11::ID3D11Device;

    pub const HEADER_API: u32 = 13 | (1 << 24);
    pub const fn sver(api: u32, n: u32) -> u32 {
        api | (n << 16) | (0x7 << 28)
    }
    pub const fn sver31(api: u32, n: u32) -> u32 {
        sver(api, n) | (1 << 31)
    }

    const DEVICE_TYPE_DIRECTX: u32 = 0;
    const RES_TYPE_DIRECTX: u32 = 0;
    const BUF_FMT_NV12: u32 = 1;
    const PIC_STRUCT_FRAME: u32 = 1;
    const PIC_TYPE_IDR: u32 = 3;
    const ENCODE_FLAG_FORCE_IDR: u32 = 0x2;
    const TUNING_HIGH_QUALITY: u32 = 1;

    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub struct Guid {
        d1: u32,
        d2: u16,
        d3: u16,
        d4: [u8; 8],
    }
    pub const G_H264: Guid = Guid { d1: 0x6bc82762, d2: 0x4e63, d3: 0x4ca4, d4: [0xaa, 0x85, 0x1e, 0x50, 0xf3, 0x21, 0xf6, 0xbf] };
    pub const G_HEVC: Guid = Guid { d1: 0x790cdc88, d2: 0x4522, d3: 0x4d7b, d4: [0x94, 0x25, 0xbd, 0xa9, 0x97, 0x5f, 0x76, 0x03] };
    pub const G_AV1: Guid = Guid { d1: 0x0a352289, d2: 0x0aa7, d3: 0x4759, d4: [0x86, 0x2d, 0x5d, 0x15, 0xcd, 0x16, 0xd2, 0x54] };
    pub const G_P1: Guid = Guid { d1: 0xfc0a8d3e, d2: 0x45f8, d3: 0x4cf8, d4: [0x80, 0xc7, 0x29, 0x88, 0x71, 0x59, 0x0e, 0xbf] };
    pub const G_P4: Guid = Guid { d1: 0x90a7b826, d2: 0xdf06, d3: 0x4862, d4: [0xb9, 0xd2, 0xcd, 0x6d, 0x73, 0xa0, 0x86, 0x81] };
    pub const G_P7: Guid = Guid { d1: 0x84848c12, d2: 0x6f71, d3: 0x4c13, d4: [0x93, 0x1b, 0x53, 0xe2, 0x83, 0xf5, 0x79, 0x74] };

    pub fn codec_guid(codec: &str) -> Guid {
        match codec {
            "hevc" => G_HEVC,
            "av1" => G_AV1,
            _ => G_H264,
        }
    }
    pub fn preset_guid(rank: u32) -> Guid {
        match rank {
            0 | 1 => G_P1,
            2 | 3 => G_P4,
            _ => G_P7,
        }
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn LoadLibraryA(name: *const u8) -> *mut c_void;
        fn GetProcAddress(module: *mut c_void, name: *const u8) -> *const c_void;
    }

    #[repr(C)]
    pub struct FuncList {
        version: u32,
        reserved: u32,
        f: [*const c_void; 43],
        _tail: [*const c_void; 275],
    }
    const I_GET_GUID_COUNT: usize = 1;
    const I_GET_GUIDS: usize = 4;
    const I_GET_INPUT_FORMAT_COUNT: usize = 5;
    const I_GET_INPUT_FORMATS: usize = 6;
    const I_GET_CAPS: usize = 7;
    const I_GET_STATS: usize = 21;
    const I_GET_SEQUENCE: usize = 22;
    const I_REGISTER_EVENT: usize = 23;
    const I_GET_PRESET_COUNT: usize = 8;
    const I_GET_PRESET_GUIDS: usize = 9;
    const I_GET_PRESET_CFG: usize = 10;
    const I_OPEN_EX: usize = 29;
    const I_INIT: usize = 11;
    const I_GET_PRESET_CFG_EX: usize = 39;
    const I_CREATE_BITSTREAM: usize = 14;
    const I_ENCODE: usize = 16;
    const I_LOCK: usize = 17;
    const I_UNLOCK: usize = 18;
    const I_MAP: usize = 25;
    const I_UNMAP: usize = 26;
    const I_DESTROY: usize = 27;
    const I_REGISTER: usize = 30;
    const I_UNREGISTER: usize = 31;
    const I_LAST_ERROR: usize = 37;

    #[repr(C)]
    struct OpenSessionEx {
        version: u32,
        device_type: u32,
        device: *mut c_void,
        reserved: *mut c_void,
        api_version: u32,
        reserved1: [u32; 253],
    }
    #[repr(C)]
    struct InitParams {
        version: u32,
        encode_guid: Guid,
        preset_guid: Guid,
        encode_width: u32,
        encode_height: u32,
        dar_width: u32,
        dar_height: u32,
        frame_rate_num: u32,
        frame_rate_den: u32,
        enable_encode_async: u32,
        enable_ptd: u32,
        bits: u32,
        priv_data_size: u32,
        reserved: u32,
        priv_data: *mut c_void,
        encode_config: *mut c_void,
        max_encode_width: u32,
        max_encode_height: u32,
        // ME hint counts: each entry is 16B (4B bitfield + 12B reserved), so [2] is 32B,
        // not 8B. Under-counting shifts later fields by 24B; the driver then reads
        // tuningInfo as 0 (UNDEFINED) and errors "Presets P1-P7 ... valid tuningInfo".
        me_hint_counts: [u32; 8],
        tuning_info: u32,
        buffer_format: u32,
        num_state_buffers: u32,
        output_stats_level: u32,
        reserved1: [u32; 284],
        reserved2: [*mut c_void; 64],
    }
    #[repr(C)]
    struct RegisterResource {
        version: u32,
        resource_type: u32,
        width: u32,
        height: u32,
        pitch: u32,
        sub_resource_index: u32,
        resource: *mut c_void,
        registered: *mut c_void,
        buffer_format: u32,
        buffer_usage: u32,
        fence: *mut c_void,
        chroma_offset: [u32; 2],
        chroma_offset_in: [u32; 2],
        reserved1: [u32; 240],
        reserved2: [*mut c_void; 60],
    }
    #[repr(C)]
    struct MapInput {
        version: u32,
        sub_resource_index: u32,
        input_resource: *mut c_void,
        registered: *mut c_void,
        mapped: *mut c_void,
        mapped_fmt: u32,
        reserved1: [u32; 251],
        reserved2: [*mut c_void; 63],
    }
    /// Bitstream buffer ring: one per frame, so we can retrieve frame N-k while submitting frame N.
    pub const RING: usize = 4;

    #[repr(C)]
    struct EventParams {
        version: u32,
        reserved: u32,
        completion_event: *mut c_void,
        reserved1: [*mut c_void; 253],
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateEventW(
            attrs: *mut c_void,
            manual_reset: i32,
            initial_state: i32,
            name: *const u16,
        ) -> *mut c_void;
    }

    #[repr(C)]
    struct CapsParam {
        version: u32,
        caps: u32,
        reserved: [u32; 62],
    }
    #[repr(C)]
    struct SeqPayload {
        version: u32,
        in_buffer_size: u32,
        sps_id: u32,
        pps_id: u32,
        spspps_buffer: *mut c_void,
        out_size: *mut u32,
        reserved: [u32; 250],
        reserved2: [*mut c_void; 64],
    }
    #[repr(C)]
    #[repr(C)]
    #[repr(C)]
    struct CreateBitstream {
        version: u32,
        size: u32,
        memory_heap: u32,
        reserved: u32,
        bitstream: *mut c_void,
        bitstream_ptr: *mut c_void,
        reserved1: [u32; 58],
        reserved2: [*mut c_void; 64],
    }
    #[repr(C)]
    struct LockBitstream {
        version: u32,
        bits: u32,
        output_bitstream: *mut c_void,
        slice_offsets: *mut u32,
        frame_idx: u32,
        hw_status: u32,
        num_slices: u32,
        size_bytes: u32,
        output_timestamp: u64,
        output_duration: u64,
        /// [out] Real bitstream data pointer. outputBitstream is an [in] buffer handle,
        /// not a data pointer — copying from it reads driver-internal memory (the stream
        /// starts with pointer bytes, ffmpeg reports "missing picture in access unit").
        bitstream_buffer_ptr: *mut c_void,
        reserved1: [u32; 62],
        reserved2: [*mut c_void; 64],
    }
    #[repr(C)]
    struct PicParams {
        version: u32,
        input_width: u32,
        input_height: u32,
        input_pitch: u32,
        encode_pic_flags: u32,
        frame_idx: u32,
        input_timestamp: u64,
        input_duration: u64,
        input_buffer: *mut c_void,
        output_bitstream: *mut c_void,
        completion_event: *mut c_void,
        buffer_fmt: u32,
        picture_struct: u32,
        picture_type: u32,
        codec_pic_params: [u32; 320],
        me_hint_counts: [u32; 2],
        me_hints: *mut c_void,
        reserved2: [u32; 7],
        reserved5: [*mut c_void; 2],
        qp_delta_map: *mut c_void,
        qp_delta_map_size: u32,
        reserved_bitfields: u32,
        me_hint_ref: [u16; 2],
        diff_pic_num_hint: i32,
        alpha_buffer: *mut c_void,
        me_sb_hints: *mut c_void,
        me_sb_hints_count: u32,
        state_buffer_idx: u32,
        output_recon: *mut c_void,
        _reserved6: [u32; 64],
        _reserved7: [*mut c_void; 64],
    }

    #[repr(align(8))]
    struct Aligned<const N: usize>([u8; N]);

    unsafe fn put_u32(b: *mut u8, off: usize, v: u32) {
        *(b.add(off) as *mut u32) = v;
    }

    fn zero<T>() -> T {
        unsafe { std::mem::zeroed() }
    }

    pub struct Encoder {
        fl: Box<FuncList>,
        api: u32,
        enc: *mut c_void,
        registered: Vec<*mut c_void>,
        bitstreams: Vec<*mut c_void>,
        maps: Vec<Option<MapInput>>,
        event: *mut c_void,
        frame_idx: u64,
        pub packets: u64,
        pub bytes: u64,
    }

    impl Encoder {
        unsafe fn last_error(&self) -> String {
            let f = self.fl.f[I_LAST_ERROR];
            if f.is_null() {
                return String::new();
            }
            let g: unsafe extern "system" fn(*mut c_void) -> *const i8 = std::mem::transmute(f);
            let p = g(self.enc);
            if p.is_null() {
                return String::new();
            }
            std::ffi::CStr::from_ptr(p)
                .to_string_lossy()
                .chars()
                .filter(|c| c.is_ascii_graphic() || *c == ' ')
                .collect()
        }
        fn fail(&self, what: &str, st: u32) -> String {
            unsafe {
                let msg = self.last_error();
                format!("{what} -> NVENCSTATUS {st:#010x} ({})", if msg.is_empty() { "-" } else { &msg })
            }
        }

        /// rc_mode: 0 = CONSTQP (CRF/cq), 1 = VBR (-b:v)
        /// rc_value: QP (0-51) for CONSTQP, average bitrate (bps) for VBR
        pub fn open(
            dev: &ID3D11Device,
            codec: &str,
            preset_rank: u32,
            w: u32,
            h: u32,
            fps: u32,
            rc_mode: u32,
            rc_value: u32,
        ) -> Result<Self, String> {
            unsafe {
                let lib = LoadLibraryA(b"nvEncodeAPI64.dll\0".as_ptr());
                if lib.is_null() {
                    return Err("LoadLibrary(nvEncodeAPI64.dll) failed".into());
                }
                let create = GetProcAddress(lib, b"NvEncodeAPICreateInstance\0".as_ptr());
                if create.is_null() {
                    return Err("NvEncodeAPICreateInstance not found".into());
                }
                // Version must come from the driver, not the header: the version word embeds
                // NVENCAPI_VERSION, and anything newer than the driver is rejected with
                // NV_ENC_ERR_INVALID_VERSION(0x0f).
                let gmv = GetProcAddress(lib, b"NvEncodeAPIGetMaxSupportedVersion\0".as_ptr());
                let mut drv = 0u32;
                if !gmv.is_null() {
                    let f: unsafe extern "system" fn(*mut u32) -> u32 = std::mem::transmute(gmv);
                    f(&mut drv);
                }
                let drv_major = (drv >> 4) & 0xf;
                let mut api = if drv_major >= 1 && drv_major <= 13 {
                    ((drv & 0xf) << 24) | drv_major
                } else {
                    HEADER_API
                };
                info!(
                    "NVENC: dll loaded, nvEncodeAPI64.dll | driver max api={}.{} -> using {}.{}",
                    drv & 0xff,
                    (drv >> 24) & 0xff,
                    api & 0xff,
                    (api >> 24) & 0xff
                );
                let create: unsafe extern "system" fn(*mut FuncList) -> u32 =
                    std::mem::transmute(create);
                let mut fl: Box<FuncList> = Box::new(zero());
                fl.version = sver(api, 2);
                let st = create(&mut *fl);
                if st != 0 {
                    return Err(format!("NvEncodeAPICreateInstance -> {st:#010x}"));
                }
                let mut me = Encoder {
                    fl,
                    api,
                    enc: std::ptr::null_mut(),
                    registered: (0..RING).map(|_| std::ptr::null_mut()).collect(),
                    bitstreams: Vec::new(),
                    maps: (0..RING).map(|_| None).collect(),
                    event: std::ptr::null_mut(),
                    frame_idx: 0,
                    packets: 0,
                    bytes: 0,
                };
                let mut sp: OpenSessionEx = zero();
                sp.version = sver(api, 1);
                sp.device_type = DEVICE_TYPE_DIRECTX;
                sp.device = dev.as_raw();
                sp.api_version = api;
                let f: unsafe extern "system" fn(*mut OpenSessionEx, *mut *mut c_void) -> u32 =
                    std::mem::transmute(me.fl.f[I_OPEN_EX]);
                let st = f(&mut sp, &mut me.enc);
                if st != 0 {
                    return Err(me.fail("nvEncOpenEncodeSessionEx", st));
                }
                info!("NVENC: session opened on D3D11 device");

                let cg = codec_guid(codec);
                let pg = preset_guid(preset_rank);

                // Ask the driver to list its codec GUIDs: this validates our function-table
                // indices (no crash = base is right) and confirms our hardcoded H264 GUID
                // matches what the driver actually uses.
                let mut n = 0u32;
                let f: unsafe extern "system" fn(*mut c_void, *mut u32) -> u32 =
                    std::mem::transmute(me.fl.f[I_GET_GUID_COUNT]);
                let st = f(me.enc, &mut n);
                if st == 0 {
                    let mut guids = [zero::<Guid>(); 8];
                    let mut got = 0u32;
                    let f: unsafe extern "system" fn(
                        *mut c_void,
                        *mut Guid,
                        u32,
                        *mut u32,
                    ) -> u32 = std::mem::transmute(me.fl.f[I_GET_GUIDS]);
                    let st2 = f(me.enc, guids.as_mut_ptr(), 8, &mut got);
                    info!(
                        "NVENC: driver reports {n} codec GUID(s), listed {got} (status {st2:#x}); first={:?} matches_h264={}",
                        guids[0],
                        got > 0 && guids[0] == cg
                    );
                } else {
                    warn!("NVENC: nvEncGetEncodeGUIDCount -> {:#010x}", st);
                }
                // Enumerate preset GUIDs the driver really supports: indices 8/9 are next to
                // the ones already verified, so this confirms our hardcoded P1 is the one the
                // driver knows, and we use it as the preferred choice.
                let mut presets: Vec<Guid> = vec![pg];
                let f: unsafe extern "system" fn(*mut c_void, Guid, *mut u32) -> u32 =
                    std::mem::transmute(me.fl.f[I_GET_PRESET_COUNT]);
                let mut pc = 0u32;
                let st = f(me.enc, cg, &mut pc);
                if st == 0 && pc > 0 {
                    let mut gs = [zero::<Guid>(); 16];
                    let mut got = 0u32;
                    let f: unsafe extern "system" fn(
                        *mut c_void,
                        Guid,
                        *mut Guid,
                        u32,
                        *mut u32,
                    ) -> u32 = std::mem::transmute(me.fl.f[I_GET_PRESET_GUIDS]);
                    let st2 = f(me.enc, cg, gs.as_mut_ptr(), 16, &mut got);
                    if st2 == 0 && got > 0 {
                        info!(
                            "NVENC: driver reports {pc} preset(s), listed {got}; first={:?} == our P1: {}",
                            gs[0],
                            gs[0] == pg
                        );
                        presets = (0..got as usize).map(|i| gs[i]).collect();
                    } else {
                        warn!("NVENC: nvEncGetEncodePresetGUIDs -> {:#010x}", st2);
                    }
                } else {
                    warn!("NVENC: nvEncGetEncodePresetCount -> {:#010x}", st);
                }

                // Fetch preset config: try both Ex (with tuningInfo) and classic variants,
                // using the GUIDs reported by the driver itself.
                let mut cfg: Aligned<8192> = std::mem::zeroed();
                let pcfg: *mut u32 = cfg.0.as_mut_ptr() as *mut u32;
                let ex: unsafe extern "system" fn(*mut c_void, Guid, Guid, u32, *mut c_void) -> u32 =
                    std::mem::transmute(me.fl.f[I_GET_PRESET_CFG_EX]);
                let classic: unsafe extern "system" fn(*mut c_void, Guid, Guid, *mut c_void) -> u32 =
                    std::mem::transmute(me.fl.f[I_GET_PRESET_CFG]);
                let cands = [
                    api,
                    13 | (1 << 24),
                    13,
                    12 | (2 << 24),
                    12,
                    11 | (4 << 24),
                    11,
                    10 | (1 << 24),
                ];
                let mut found: Option<(u32, Guid)> = None;
                let mut cfg_ok = false;
                let mut tried = String::new();
                'outer: for g in presets.iter().take(3) {
                    for c in cands {
                        *pcfg = sver31(c, 5);
                        let st = ex(me.enc, cg, *g, TUNING_HIGH_QUALITY, pcfg as *mut c_void);
                        if st == 0 {
                            found = Some((c, *g));
                            break 'outer;
                        }
                        if tried.len() < 600 {
                            tried.push_str(&format!("ex{}.{}:g{:x}={st:x} ", c & 0xff, (c >> 24) & 0xff, g.d1));
                        }
                    }
                    for c in cands {
                        *pcfg = sver31(c, 5);
                        let st = classic(me.enc, cg, *g, pcfg as *mut c_void);
                        if st == 0 {
                            found = Some((c, *g));
                            break 'outer;
                        }
                        if tried.len() < 600 {
                            tried.push_str(&format!("cl{}.{}:g{:x}={st:x} ", c & 0xff, (c >> 24) & 0xff, g.d1));
                        }
                    }
                }
                match found {
                    Some((c, g)) => {
                        api = c;
                        me.api = c;
                        cfg_ok = true;
                        info!(
                            "NVENC: preset config OK api={}.{} preset={:#x}",
                            c & 0xff,
                            (c >> 24) & 0xff,
                            g.d1
                        );
                    }
                    None => {
                        // Fallback: pass encodeConfig=NULL and only presetGUID. The header says
                        // "if the preset GUID is set then the preset configuration will be applied
                        // before any other parameter", so this path may work by itself.
                        warn!("NVENC: preset config unavailable [{}] (drv_raw={:#x}); trying encodeConfig=NULL + presetGUID", tried, drv);
                    }
                }
                let off = (0..8usize)
                    .find(|i| *(cfg.0.as_ptr().add(i * 4) as *const u32) == sver31(api, 9));
                let off = match off {
                    Some(i) => i * 4,
                    None => {
                        warn!("NVENC: preset config version not found in buffer, assuming offset 8");
                        8
                    }
                };
                info!("NVENC: preset config filled at offset {}", off);
                let mut manual: Aligned<4096> = std::mem::zeroed();
                if !cfg_ok {
                    let b = manual.0.as_mut_ptr();
                    unsafe {
                        put_u32(b, 0, sver31(api, 9)); // NV_ENC_CONFIG_VER
                        put_u32(b, 20, fps * 2); // gopLength
                        put_u32(b, 24, 1); // frameIntervalP = 1 (no B frames!)
                        put_u32(b, 32, 1); // frameFieldMode = FRAME
                        put_u32(b, 36, 0); // mvPrecision = DEFAULT
                        put_u32(b, 40, sver(api, 1)); // NV_ENC_RC_PARAMS_VER
                        put_u32(b, 44, rc_mode); // 0=CONSTQP(CRF) / 1=VBR(bitrate)
                        if rc_mode == 0 {
                            // constant QP: qpInterP / qpInterB / qpIntra
                            put_u32(b, 48, rc_value);
                            put_u32(b, 52, rc_value);
                            put_u32(b, 56, rc_value);
                        } else {
                            put_u32(b, 60, rc_value); // averageBitRate
                            put_u32(b, 64, rc_value); // maxBitRate
                        }
                        let cc = 40 + 128; // encodeCodecConfig
                        put_u32(b, cc + 8, fps * 2); // h264Config.idrPeriod
                        put_u32(b, cc + 44, 1); // entropyCodingMode = CABAC
                        // offset = 72 (H264 header fields) + 112 (VUI: 15 scalars + reserved[12]) + 8 (ltrNumFrames/ltrTrustMode)
                        put_u32(b, cc + 192, 1);
                    }
                    info!(
                        "NVENC: hand-built NV_ENC_CONFIG (gop={}, {}, CABAC)",
                        fps * 2,
                        if rc_mode == 0 {
                            format!("CONSTQP qp={rc_value}")
                        } else {
                            format!("VBR {} kbps", rc_value / 1000)
                        }
                    );
                }

                let mut ip: InitParams = zero();
                ip.version = sver31(api, 7);
                ip.encode_guid = cg;
                ip.preset_guid = pg;
                ip.encode_width = w;
                ip.encode_height = h;
                ip.dar_width = w;
                ip.dar_height = h;
                ip.frame_rate_num = fps;
                ip.frame_rate_den = 1;
                ip.enable_ptd = 1;
                // Async encoding: sync mode is capped by single-frame latency (measured only
                // 708 fps at 1080p). Async + bitstream ring lets encoding overlap rendering,
                // like ffmpeg.
                ip.enable_encode_async = 1;
                ip.encode_config = if cfg_ok {
                    cfg.0.as_mut_ptr().add(off) as *mut c_void
                } else {
                    manual.0.as_mut_ptr() as *mut c_void
                };
                ip.max_encode_width = w;
                ip.max_encode_height = h;
                ip.tuning_info = TUNING_HIGH_QUALITY;
                ip.buffer_format = BUF_FMT_NV12;
                let f: unsafe extern "system" fn(*mut c_void, *mut InitParams) -> u32 =
                    std::mem::transmute(me.fl.f[I_INIT]);
                let st = f(me.enc, &mut ip);
                if st != 0 {
                    return Err(me.fail("nvEncInitializeEncoder", st));
                }
                info!("NVENC: encoder initialized {}x{} @{} fps, codec={codec}", w, h, fps);

                let ev = CreateEventW(std::ptr::null_mut(), 0, 0, std::ptr::null());
                if ev.is_null() {
                    return Err("CreateEventW failed".into());
                }
                me.event = ev;
                let mut ep = EventParams {
                    version: sver(api, 2),
                    reserved: 0,
                    completion_event: ev,
                    reserved1: [std::ptr::null_mut(); 253],
                };
                let f: unsafe extern "system" fn(*mut c_void, *mut EventParams) -> u32 =
                    std::mem::transmute(me.fl.f[I_REGISTER_EVENT]);
                let st = f(me.enc, &mut ep);
                if st != 0 {
                    return Err(me.fail("nvEncRegisterAsyncEvent", st));
                }

                for slot in 0..RING {
                    let mut cb: CreateBitstream = zero();
                    cb.version = sver(api, 1);
                    let f: unsafe extern "system" fn(*mut c_void, *mut CreateBitstream) -> u32 =
                        std::mem::transmute(me.fl.f[I_CREATE_BITSTREAM]);
                    let st = f(me.enc, &mut cb);
                    if st != 0 {
                        return Err(me.fail(&format!("nvEncCreateBitstreamBuffer[{slot}]"), st));
                    }
                    me.bitstreams.push(cb.bitstream);
                }
                info!("NVENC: async mode, {} bitstream buffers", RING);
                Ok(me)
            }
        }

        pub fn register_nv12(
            &mut self,
            slot: usize,
            cg: Guid,
            tex: &windows::Win32::Graphics::Direct3D11::ID3D11Texture2D,
            w: u32,
            h: u32,
        ) -> Result<(), String> {
            unsafe {
                let mut rr: RegisterResource = zero();
                rr.version = sver(self.api, 5);
                rr.resource_type = RES_TYPE_DIRECTX;
                rr.width = w;
                rr.height = h;
                rr.resource = tex.as_raw();
                rr.buffer_format = BUF_FMT_NV12;
                let f: unsafe extern "system" fn(*mut c_void, *mut RegisterResource) -> u32 =
                    std::mem::transmute(self.fl.f[I_REGISTER]);
                let st = f(self.enc, &mut rr);
                if st != 0 {
                    return Err(self.fail("nvEncRegisterResource", st));
                }
                self.registered[slot] = rr.registered;
                if slot == 0 {
                    info!(
                        "NVENC: NV12 texture registered as input resource | {}",
                        self.diagnostics(cg)
                    );
                }
                Ok(())
            }
        }

        /// Submit one frame (async: returns right away, hardware encodes in the background)
        pub fn submit(&mut self, slot: usize, force_idr: bool, w: u32, h: u32) -> Result<(), String> {
            unsafe {
                if let Some(mut old) = self.maps[slot].take() {
                    let f: unsafe extern "system" fn(*mut c_void, *mut MapInput) -> u32 =
                        std::mem::transmute(self.fl.f[I_UNMAP]);
                    f(self.enc, &mut old);
                }
                let mut mi: MapInput = zero();
                mi.version = sver(self.api, 4);
                mi.registered = self.registered[slot];
                let f: unsafe extern "system" fn(*mut c_void, *mut MapInput) -> u32 =
                    std::mem::transmute(self.fl.f[I_MAP]);
                let st = f(self.enc, &mut mi);
                if st != 0 {
                    return Err(self.fail("nvEncMapInputResource", st));
                }

                let mut pp: PicParams = zero();
                pp.version = sver31(self.api, 7);
                pp.input_width = w;
                pp.input_height = h;
                pp.input_pitch = (w + 255) & !255;
                // 0x2 = FORCEIDR; 0x4 = OUTPUT_SPSPPS — a raw stream must carry its own SPS/PPS,
                // otherwise ffmpeg's h264 demuxer cannot parse it and exits (pipe closing 232).
                pp.encode_pic_flags = if force_idr { ENCODE_FLAG_FORCE_IDR | 0x4 } else { 0 };
                pp.frame_idx = self.frame_idx as u32;
                pp.input_timestamp = self.frame_idx * 166_666;
                pp.input_duration = 166_666;
                pp.input_buffer = mi.mapped;
                pp.output_bitstream = self.bitstreams[slot];
                pp.completion_event = self.event;
                pp.buffer_fmt = mi.mapped_fmt;
                pp.picture_struct = PIC_STRUCT_FRAME;
                pp.picture_type = if force_idr { PIC_TYPE_IDR } else { 0 };
                let f: unsafe extern "system" fn(*mut c_void, *mut PicParams) -> u32 =
                    std::mem::transmute(self.fl.f[I_ENCODE]);
                let st = f(self.enc, &mut pp);
                if st != 0 {
                    return Err(format!(
                        "{} [w={} h={} pitch={} fmt={} flags={:#x} ptype={} buf={:?}]",
                        self.fail("nvEncEncodePicture", st),
                        pp.input_width, pp.input_height, pp.input_pitch, pp.buffer_fmt,
                        pp.encode_pic_flags, pp.picture_type, pp.input_buffer
                    ));
                }
                self.maps[slot] = Some(mi);
                self.frame_idx += 1;
                Ok(())
            }
        }

        /// Retrieve the bitstream of a slot (async: waits until that frame is done)
        pub fn retrieve(&mut self, slot: usize) -> Result<Vec<u8>, String> {
            unsafe {
                let mut lb: LockBitstream = zero();
                lb.version = sver31(self.api, 2);
                lb.output_bitstream = self.bitstreams[slot];
                let f: unsafe extern "system" fn(*mut c_void, *mut LockBitstream) -> u32 =
                    std::mem::transmute(self.fl.f[I_LOCK]);
                let st = f(self.enc, &mut lb);
                if st != 0 {
                    return Err(format!(
                        "{} [ver={:#010x} bitstream={:?} slices={} size={}]",
                        self.fail("nvEncLockBitstream", st),
                        lb.version, lb.output_bitstream, lb.num_slices, lb.size_bytes
                    ));
                }
                let data = if lb.size_bytes > 0 && !lb.bitstream_buffer_ptr.is_null() {
                    std::slice::from_raw_parts(
                        lb.bitstream_buffer_ptr as *const u8,
                        lb.size_bytes as usize,
                    )
                    .to_vec()
                } else {
                    Vec::new()
                };
                let f: unsafe extern "system" fn(*mut c_void, *mut c_void) -> u32 =
                    std::mem::transmute(self.fl.f[I_UNLOCK]);
                f(self.enc, lb.output_bitstream);
                if let Some(mut mi) = self.maps[slot].take() {
                    let f: unsafe extern "system" fn(*mut c_void, *mut MapInput) -> u32 =
                        std::mem::transmute(self.fl.f[I_UNMAP]);
                    f(self.enc, &mut mi);
                }
                self.packets += 1;
                self.bytes += data.len() as u64;
                Ok(data)
            }
        }

        /// Sync convenience API (for self-test): submit then retrieve immediately
        pub fn encode(&mut self, force_idr: bool, w: u32, h: u32) -> Result<Vec<u8>, String> {
            self.submit(0, force_idr, w, h)?;
            self.retrieve(0)
        }

        /// Diagnostics: ask the driver for this encoder's caps and input format list.
        pub fn diagnostics(&self, cg: Guid) -> String {
            unsafe {
                let caps = |c: u32| -> i32 {
                    let mut p = CapsParam {
                        version: sver(self.api, 1),
                        caps: c,
                        reserved: [0; 62],
                    };
                    let f: unsafe extern "system" fn(*mut c_void, Guid, *mut CapsParam, *mut i32) -> u32 =
                        std::mem::transmute(self.fl.f[I_GET_CAPS]);
                    let mut out = -1i32;
                    if f(self.enc, cg, &mut p, &mut out) != 0 {
                        return -1;
                    }
                    out
                };
                let mut n = 0u32;
                let f: unsafe extern "system" fn(*mut c_void, Guid, *mut u32) -> u32 =
                    std::mem::transmute(self.fl.f[I_GET_INPUT_FORMAT_COUNT]);
                let mut fmts = String::from("?");
                if f(self.enc, cg, &mut n) == 0 && n > 0 {
                    let mut list = vec![0u32; n as usize];
                    let mut got = 0u32;
                    let f: unsafe extern "system" fn(
                        *mut c_void,
                        Guid,
                        *mut u32,
                        u32,
                        *mut u32,
                    ) -> u32 = std::mem::transmute(self.fl.f[I_GET_INPUT_FORMATS]);
                    if f(self.enc, cg, list.as_mut_ptr(), n, &mut got) == 0 {
                        list.truncate(got as usize);
                        fmts = format!("{list:?} (NV12=1 included: {})", list.contains(&1));
                    }
                }
                let mut sp = [0u8; 1024];
                let mut out_size = 0u32;
                let mut payload = SeqPayload {
                    version: sver(self.api, 1),
                    in_buffer_size: sp.len() as u32,
                    sps_id: 0,
                    pps_id: 0,
                    spspps_buffer: sp.as_mut_ptr() as *mut c_void,
                    out_size: &mut out_size,
                    reserved: [0; 250],
                    reserved2: [std::ptr::null_mut(); 64],
                };
                let f: unsafe extern "system" fn(*mut c_void, *mut SeqPayload) -> u32 =
                    std::mem::transmute(self.fl.f[I_GET_SEQUENCE]);
                let seq_st = f(self.enc, &mut payload);
                format!(
                    "caps[w_max={} h_max={} mb_max={} cabac={} rcmodes={:#x} bframes_max={} async={}] input_fmts={} spspps={:#x}({} B)",
                    caps(16),
                    caps(17),
                    caps(31),
                    caps(7),
                    caps(1),
                    caps(0),
                    caps(30),
                    fmts,
                    seq_st,
                    out_size
                )
            }
        }

        pub fn stats(&self) -> String {
            format!("{} frames / {} bytes", self.packets, self.bytes)
        }
    }

    impl Drop for Encoder {
        fn drop(&mut self) {
            unsafe {
                let f: unsafe extern "system" fn(*mut c_void, *mut c_void) -> u32 =
                    std::mem::transmute(self.fl.f[I_UNREGISTER]);
                for r in &self.registered {
                    if !r.is_null() {
                        f(self.enc, *r);
                    }
                }
                if !self.enc.is_null() {
                    let f: unsafe extern "system" fn(*mut c_void) -> u32 =
                        std::mem::transmute(self.fl.f[I_DESTROY]);
                    f(self.enc);
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub fn probe_zero_copy_support() {
    use std::ffi::{c_void, CStr};

    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleA(name: *const u8) -> *mut c_void;
        fn GetProcAddress(module: *mut c_void, name: *const u8) -> *const c_void;
    }

    let ext = unsafe {
        let opengl = GetModuleHandleA(b"opengl32.dll\0".as_ptr());
        let get_dc = GetProcAddress(opengl, b"wglGetCurrentDC\0".as_ptr());
        let get_proc = GetProcAddress(opengl, b"wglGetProcAddress\0".as_ptr());
        if opengl.is_null() || get_dc.is_null() || get_proc.is_null() {
            warn!("Zero-copy probe: opengl32/wglGetProcAddress unavailable");
            String::new()
        } else {
            // WGL extension functions are not exported by opengl32; they must be loaded
            // through wglGetProcAddress.
            let wgl_get_proc: unsafe extern "system" fn(*const i8) -> *const c_void =
                std::mem::transmute(get_proc);
            let wgl_get_dc: unsafe extern "system" fn() -> *mut c_void =
                std::mem::transmute(get_dc);
            let dc = wgl_get_dc();

            let arb = wgl_get_proc(b"wglGetExtensionsStringARB\0".as_ptr() as *const i8);
            let s = if !arb.is_null() {
                let f: unsafe extern "system" fn(*mut c_void) -> *const i8 =
                    std::mem::transmute(arb);
                f(dc)
            } else {
                let extf = wgl_get_proc(b"wglGetExtensionsStringEXT\0".as_ptr() as *const i8);
                if extf.is_null() {
                    warn!("Zero-copy probe: no wglGetExtensionsStringARB/EXT");
                    std::ptr::null()
                } else {
                    let f: unsafe extern "system" fn() -> *const i8 = std::mem::transmute(extf);
                    f()
                }
            };
            if s.is_null() {
                warn!("Zero-copy probe: WGL extension string is null (no current context?)");
                String::new()
            } else {
                CStr::from_ptr(s).to_string_lossy().into_owned()
            }
        }
    };

    let sys = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_owned());
    let dll = |n: &str| std::path::Path::new(&sys).join("System32").join(n).exists();

    info!(
        "Zero-copy probe: WGL_NV_DX_interop2={} WGL_NV_DX_interop={} nvEncodeAPI64={} mfplat={} d3d11={} ({:.0} WGL ext)",
        ext.contains("WGL_NV_DX_interop2"),
        ext.contains("WGL_NV_DX_interop"),
        dll("nvEncodeAPI64.dll"),
        dll("mfplat.dll"),
        dll("d3d11.dll"),
        ext.split_whitespace().count() as f64
    );
    if std::env::var("PHITK_DUMP_WGL").is_ok() {
        info!("WGL extensions: {}", ext);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn probe_zero_copy_support() {}
#[cfg(target_os = "windows")]
pub mod zerocopy {
    use super::*;
    use std::ffi::c_void;
    use windows::core::Interface;
    use windows::Win32::Foundation::HMODULE;
    use windows::Win32::Graphics::Direct3D::{
        D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST, D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0,
        Fxc::D3DCompile, ID3DBlob,
    };
    use windows::Win32::Graphics::Direct3D11::*;
    use windows::Win32::Graphics::Dxgi::Common::{
        DXGI_FORMAT, DXGI_FORMAT_NV12, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_R8G8_UNORM,
        DXGI_FORMAT_R8_UNORM, DXGI_SAMPLE_DESC,
    };

    const HLSL: &str = r#"
Texture2D tex : register(t0);
SamplerState smp : register(s0);
struct VSOut { float4 pos : SV_POSITION; float2 uv : TEXCOORD0; };
VSOut VSMain(uint id : SV_VertexID) {
    VSOut o;
    float2 p = float2((id << 1) & 2, id & 2);
    o.pos = float4(p * float2(2.0, -2.0) + float2(-1.0, 1.0), 0.0, 1.0);
    o.uv = float2(p.x, 1.0 - p.y);
    return o;
}
float4 PSY(VSOut i) : SV_Target {
    float3 c = tex.Sample(smp, i.uv).rgb;
    float y = (16.0 + 219.0 * (0.299 * c.r + 0.587 * c.g + 0.114 * c.b)) / 255.0;
    return float4(y, 0.0, 0.0, 1.0);
}
float4 PSUV(VSOut i) : SV_Target {
    float3 c = tex.Sample(smp, i.uv).rgb;
    float u = (128.0 + 224.0 * (-0.168736 * c.r - 0.331264 * c.g + 0.5 * c.b)) / 255.0;
    float v = (128.0 + 224.0 * (0.5 * c.r - 0.418688 * c.g - 0.081312 * c.b)) / 255.0;
    return float4(u, v, 0.0, 1.0);
}
"#;

    fn compile(src: &str, entry: &str, target: &str) -> Result<ID3DBlob, String> {
        let entry = std::ffi::CString::new(entry).unwrap();
        let target = std::ffi::CString::new(target).unwrap();
        let mut blob = None;
        let mut err = None;
        unsafe {
            D3DCompile(
                src.as_ptr() as *const _,
                src.len(),
                None,
                None,
                None,
                windows::core::PCSTR(entry.as_ptr() as *const u8),
                windows::core::PCSTR(target.as_ptr() as *const u8),
                0,
                0,
                &mut blob,
                Some(&mut err),
            )
        }
        .map_err(|e| {
            let msg = err
                .map(|b| unsafe {
                    std::slice::from_raw_parts(b.GetBufferPointer() as *const u8, b.GetBufferSize())
                }
                .iter()
                .map(|c| *c as char)
                .collect::<String>())
                .unwrap_or_default();
            format!("D3DCompile({target:?}): {e} {msg}")
        })?;
        blob.ok_or_else(|| format!("D3DCompile({target:?}) produced no blob"))
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleA(name: *const u8) -> *mut c_void;
        fn GetProcAddress(module: *mut c_void, name: *const u8) -> *const c_void;
    }

    type FnOpen = unsafe extern "system" fn(*mut c_void) -> *mut c_void;
    type FnRegister =
        unsafe extern "system" fn(*mut c_void, *mut c_void, u32, u32, u32) -> *mut c_void;
    type FnTwo = unsafe extern "system" fn(*mut c_void, i32, *mut *mut c_void) -> i32;
    type FnOne = unsafe extern "system" fn(*mut c_void, *mut c_void) -> i32;
    type FnClose = unsafe extern "system" fn(*mut c_void) -> i32;

    const GL_TEXTURE_2D: u32 = 0x0DE1;
    const WGL_ACCESS_WRITE_DISCARD_NV: u32 = 0x0002;

    struct Wgl {
        lock: FnTwo,
        unlock: FnTwo,
        unregister: FnOne,
        close: FnClose,
        interop: *mut c_void,
        obj: *mut c_void,
    }

    /// Persistent zero-copy pipeline: GL scene target ←(wglDXRegisterObjectNV)→ D3D11 texture
    ///  → D3D pixel shader converts to NV12 → NVENC encodes.
    /// Built once only when the encoder is nvenc + self-test passes + PHITK_NO_ZEROCOPY unset.
    pub struct ZeroCopy {
        dev: ID3D11Device,
        ctx: ID3D11DeviceContext,
        wgl: Wgl,
        /// Ring buffer: each slot has its own GL target / D3D texture / interop object / NV12 texture
        slots: Vec<Slot>,
        vs: ID3D11VertexShader,
        ps_y: ID3D11PixelShader,
        ps_uv: ID3D11PixelShader,
        smp: ID3D11SamplerState,
        enc: nvenc::Encoder,
        pub w: u32,
        pub h: u32,
        pub frames: u64,
        pub bytes: u64,
    }

    struct Slot {
        target: RenderTarget,
        src: ID3D11Texture2D,
        srv: ID3D11ShaderResourceView,
        obj: *mut c_void,
        nv12: ID3D11Texture2D,
        rtv_y: ID3D11RenderTargetView,
        rtv_uv: ID3D11RenderTargetView,
    }

    impl ZeroCopy {
        /// Create RING slots: each is "GL target ↔ D3D texture ↔ interop object + NV12 texture".
        /// The ring is required for async encoding: while the driver still reads slot A,
        /// rendering can write slot B.
        pub fn new(
            codec: &str,
            w: u32,
            h: u32,
            fps: u32,
            preset_rank: u32,
            rc_mode: u32,
            rc_value: u32,
        ) -> Result<Self, String> {
            unsafe {
                let mut dev = None;
                let mut ctx = None;
                D3D11CreateDevice(
                    None,
                    D3D_DRIVER_TYPE_HARDWARE,
                    HMODULE::default(),
                    D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_VIDEO_SUPPORT,
                    Some(&[D3D_FEATURE_LEVEL_11_0]),
                    D3D11_SDK_VERSION,
                    Some(&mut dev),
                    None,
                    Some(&mut ctx),
                )
                .map_err(|e| format!("D3D11CreateDevice failed: {e}"))?;
                let dev = dev.ok_or("D3D11CreateDevice returned no device")?;
                let ctx = ctx.ok_or("D3D11CreateDevice returned no context")?;
                match dev.cast::<windows::Win32::Graphics::Direct3D10::ID3D10Multithread>() {
                    Ok(mt) => {
                        let _ = mt.SetMultithreadProtected(true);
                    }
                    Err(e) => warn!("NVENC: ID3D10Multithread unavailable: {}", e),
                }

                let mk_desc = |format: DXGI_FORMAT| D3D11_TEXTURE2D_DESC {
                    Width: w,
                    Height: h,
                    MipLevels: 1,
                    ArraySize: 1,
                    Format: format,
                    SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                    Usage: D3D11_USAGE_DEFAULT,
                    BindFlags: (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32,
                    CPUAccessFlags: 0,
                    MiscFlags: 0,
                };

                let opengl = GetModuleHandleA(b"opengl32.dll\0".as_ptr());
                let get_proc = GetProcAddress(opengl, b"wglGetProcAddress\0".as_ptr());
                if get_proc.is_null() {
                    return Err("wglGetProcAddress missing".into());
                }
                let wgl_get_proc: unsafe extern "system" fn(*const i8) -> *const c_void =
                    std::mem::transmute(get_proc);
                let sym = |n: &[u8]| wgl_get_proc(n.as_ptr() as *const i8);
                let open = sym(b"wglDXOpenDeviceNV\0");
                let register = sym(b"wglDXRegisterObjectNV\0");
                let lockp = sym(b"wglDXLockObjectsNV\0");
                let unlockp = sym(b"wglDXUnlockObjectsNV\0");
                let unregisterp = sym(b"wglDXUnregisterObjectNV\0");
                let closep = sym(b"wglDXCloseDeviceNV\0");
                if open.is_null() || register.is_null() || lockp.is_null() || unlockp.is_null() {
                    return Err("WGL_NV_DX_interop entry points missing".into());
                }
                let interop = std::mem::transmute::<_, FnOpen>(open)(dev.as_raw());
                if interop.is_null() {
                    return Err("wglDXOpenDeviceNV failed".into());
                }

                let bytes = |b: &ID3DBlob| unsafe {
                    std::slice::from_raw_parts(b.GetBufferPointer() as *const u8, b.GetBufferSize())
                };
                let vs_blob = compile(HLSL, "VSMain", "vs_5_0")?;
                let py_blob = compile(HLSL, "PSY", "ps_5_0")?;
                let pu_blob = compile(HLSL, "PSUV", "ps_5_0")?;
                let mut vs = None;
                dev.CreateVertexShader(bytes(&vs_blob), None, Some(&mut vs))
                    .map_err(|e| format!("CreateVertexShader: {e}"))?;
                let mut ps_y = None;
                dev.CreatePixelShader(bytes(&py_blob), None, Some(&mut ps_y))
                    .map_err(|e| format!("CreatePixelShader(Y): {e}"))?;
                let mut ps_uv = None;
                dev.CreatePixelShader(bytes(&pu_blob), None, Some(&mut ps_uv))
                    .map_err(|e| format!("CreatePixelShader(UV): {e}"))?;
                let smp_desc = D3D11_SAMPLER_DESC {
                    Filter: D3D11_FILTER_MIN_MAG_MIP_LINEAR,
                    AddressU: D3D11_TEXTURE_ADDRESS_CLAMP,
                    AddressV: D3D11_TEXTURE_ADDRESS_CLAMP,
                    AddressW: D3D11_TEXTURE_ADDRESS_CLAMP,
                    MipLODBias: 0.0,
                    MaxAnisotropy: 1,
                    ComparisonFunc: D3D11_COMPARISON_NEVER,
                    BorderColor: [0.0; 4],
                    MinLOD: 0.0,
                    MaxLOD: f32::MAX,
                };
                let mut smp = None;
                dev.CreateSamplerState(&smp_desc, Some(&mut smp))
                    .map_err(|e| format!("CreateSamplerState: {e}"))?;

                let mut slots = Vec::with_capacity(nvenc::RING);
                for i in 0..nvenc::RING {
                    let target = new_plane(w, h, TextureFormat::RGBA8);
                    let gl_id = target.texture.raw_miniquad_texture_handle().gl_internal_id();
                    let mut src = None;
                    dev.CreateTexture2D(&mk_desc(DXGI_FORMAT_R8G8B8A8_UNORM), None, Some(&mut src))
                        .map_err(|e| format!("CreateTexture2D(rgba[{i}]) failed: {e}"))?;
                    let src = src.ok_or("no rgba texture")?;
                    let obj = std::mem::transmute::<_, FnRegister>(register)(
                        interop,
                        src.as_raw(),
                        gl_id,
                        GL_TEXTURE_2D,
                        WGL_ACCESS_WRITE_DISCARD_NV,
                    );
                    if obj.is_null() {
                        return Err(format!("wglDXRegisterObjectNV failed (slot {i})"));
                    }
                    let mut nv12 = None;
                    dev.CreateTexture2D(&mk_desc(DXGI_FORMAT_NV12), None, Some(&mut nv12))
                        .map_err(|e| format!("CreateTexture2D(nv12[{i}]) failed: {e}"))?;
                    let nv12 = nv12.ok_or("no nv12 texture")?;
                    let plane_rtv = |fmt: DXGI_FORMAT| -> Result<ID3D11RenderTargetView, String> {
                        let rtv_desc = D3D11_RENDER_TARGET_VIEW_DESC {
                            Format: fmt,
                            ViewDimension: D3D11_RTV_DIMENSION_TEXTURE2D,
                            Anonymous: D3D11_RENDER_TARGET_VIEW_DESC_0 {
                                Texture2D: D3D11_TEX2D_RTV { MipSlice: 0 },
                            },
                        };
                        let mut rtv = None;
                        dev.CreateRenderTargetView(&nv12, Some(&rtv_desc), Some(&mut rtv))
                            .map_err(|e| format!("plane RTV({fmt:?}) failed: {e}"))?;
                        rtv.ok_or_else(|| format!("plane RTV({fmt:?}) null"))
                    };
                    let rtv_y = plane_rtv(DXGI_FORMAT_R8_UNORM)?;
                    let rtv_uv = plane_rtv(DXGI_FORMAT_R8G8_UNORM)?;
                    let mut srv = None;
                    dev.CreateShaderResourceView(&src, None, Some(&mut srv))
                        .map_err(|e| format!("CreateShaderResourceView({i}): {e}"))?;
                    slots.push(Slot {
                        target,
                        src,
                        srv: srv.ok_or("no srv")?,
                        obj,
                        nv12,
                        rtv_y,
                        rtv_uv,
                    });
                }

                let enc =
                    nvenc::Encoder::open(&dev, codec, preset_rank, w, h, fps, rc_mode, rc_value)?;
                let mut zc = ZeroCopy {
                    dev,
                    ctx,
                    wgl: Wgl {
                        lock: std::mem::transmute(lockp),
                        unlock: std::mem::transmute(unlockp),
                        unregister: std::mem::transmute(unregisterp),
                        close: std::mem::transmute(closep),
                        interop,
                        obj: std::ptr::null_mut(),
                    },
                    slots,
                    vs: vs.ok_or("no vs")?,
                    ps_y: ps_y.ok_or("no ps_y")?,
                    ps_uv: ps_uv.ok_or("no ps_uv")?,
                    smp: smp.ok_or("no sampler")?,
                    enc,
                    w,
                    h,
                    frames: 0,
                    bytes: 0,
                };
                for i in 0..nvenc::RING {
                    zc.enc
                        .register_nv12(i, nvenc::codec_guid(codec), &zc.slots[i].nv12, w, h)?;
                }
                Ok(zc)
            }
        }

        pub fn ring() -> usize {
            nvenc::RING
        }

        pub fn lock(&self, slot: usize) -> Result<(), String> {
            let mut objs = [self.slots[slot].obj];
            let ok = unsafe { (self.wgl.lock)(self.wgl.interop, 1, objs.as_mut_ptr()) };
            if ok == 0 {
                return Err("wglDXLockObjectsNV failed".into());
            }
            Ok(())
        }

        pub fn unlock(&self, slot: usize) {
            unsafe {
                use miniquad::gl::*;
                // Flush only, not finish: after unlock the driver inserts a fence on the D3D
                // side to sync. glFinish would stall the GPU every frame (measured at 78% of
                // total time).
                glFlush();
                let mut objs = [self.slots[slot].obj];
                (self.wgl.unlock)(self.wgl.interop, 1, objs.as_mut_ptr());
            }
        }

        /// Blit the source render target into this slot (GL-side blit, same size and format)
        pub fn blit_from(&self, slot: usize, src: &RenderTarget) {
            unsafe {
                use miniquad::gl::*;
                glBindFramebuffer(GL_READ_FRAMEBUFFER, internal_id(src));
                glBindFramebuffer(GL_DRAW_FRAMEBUFFER, internal_id(&self.slots[slot].target));
                glBlitFramebuffer(
                    0,
                    0,
                    self.w as i32,
                    self.h as i32,
                    0,
                    0,
                    self.w as i32,
                    self.h as i32,
                    GL_COLOR_BUFFER_BIT,
                    GL_NEAREST,
                );
                glBindFramebuffer(GL_FRAMEBUFFER, 0);
            }
        }

        /// On the D3D side, convert this slot's RGBA to NV12
        fn convert(&self, slot: usize) {
            unsafe {
                let s = &self.slots[slot];
                let mut pass = |rtv: &ID3D11RenderTargetView,
                                ps: &ID3D11PixelShader,
                                w: f32,
                                h: f32| {
                    self.ctx.OMSetRenderTargets(Some(&[Some(rtv.clone())]), None);
                    self.ctx.VSSetShader(&self.vs, None);
                    self.ctx.PSSetShader(ps, None);
                    self.ctx
                        .PSSetShaderResources(0, Some(&[Some(s.srv.clone())]));
                    self.ctx.PSSetSamplers(0, Some(&[Some(self.smp.clone())]));
                    self.ctx
                        .IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
                    self.ctx.RSSetViewports(Some(&[D3D11_VIEWPORT {
                        TopLeftX: 0.0,
                        TopLeftY: 0.0,
                        Width: w,
                        Height: h,
                        MinDepth: 0.0,
                        MaxDepth: 1.0,
                    }]));
                    self.ctx.Draw(3, 0);
                };
                pass(&s.rtv_y, &self.ps_y, self.w as f32, self.h as f32);
                pass(&s.rtv_uv, &self.ps_uv, (self.w / 2) as f32, (self.h / 2) as f32);
            }
        }
        
        pub fn submit(&mut self, slot: usize, force_idr: bool) -> Result<(), String> {
            self.convert(slot);
            self.enc.submit(slot, force_idr, self.w, self.h)?;
            self.frames += 1;
            Ok(())
        }

        pub fn retrieve(&mut self, slot: usize) -> Result<Vec<u8>, String> {
            let d = self.enc.retrieve(slot)?;
            self.bytes += d.len() as u64;
            Ok(d)
        }

        pub fn selftest_rgba(&self) -> Result<[u8; 4], String> {
            unsafe {
                self.lock(0)?;
                {
                    use miniquad::gl::*;
                    glBindFramebuffer(GL_FRAMEBUFFER, internal_id(&self.slots[0].target));
                    glClearColor(1.0, 0.0, 1.0, 1.0);
                    glClear(GL_COLOR_BUFFER_BIT);
                }
                self.unlock(0);
                let stage = self.staging(DXGI_FORMAT_R8G8B8A8_UNORM)?;
                self.ctx.CopyResource(&stage, &self.slots[0].src);
                let mut m = D3D11_MAPPED_SUBRESOURCE::default();
                self.ctx
                    .Map(&stage, 0, D3D11_MAP_READ, 0, Some(&mut m))
                    .map_err(|e| format!("Map failed: {e}"))?;
                let px = std::slice::from_raw_parts(m.pData as *const u8, 4);
                let got = [px[0], px[1], px[2], px[3]];
                self.ctx.Unmap(&stage, 0);
                Ok(got)
            }
        }

        /// Self-test: read Y/U/V back from the NV12 planes after conversion (uses slot 0)
        pub fn selftest_nv12(&self) -> Result<[u8; 3], String> {
            unsafe {
                self.convert(0);
                let stage = self.staging(DXGI_FORMAT_NV12)?;
                self.ctx.CopyResource(&stage, &self.slots[0].nv12);
                let mut m = D3D11_MAPPED_SUBRESOURCE::default();
                self.ctx
                    .Map(&stage, 0, D3D11_MAP_READ, 0, Some(&mut m))
                    .map_err(|e| format!("NV12 Map failed: {e}"))?;
                let y = *(m.pData as *const u8);
                let uv_row = (m.pData as *const u8).add(self.h as usize * m.RowPitch as usize);
                let u = *uv_row;
                let v = *uv_row.add(1);
                self.ctx.Unmap(&stage, 0);
                Ok([y, u, v])
            }
        }

        fn staging(&self, format: DXGI_FORMAT) -> Result<ID3D11Texture2D, String> {
            let desc = D3D11_TEXTURE2D_DESC {
                Width: self.w,
                Height: self.h,
                MipLevels: 1,
                ArraySize: 1,
                Format: format,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                Usage: D3D11_USAGE_STAGING,
                BindFlags: 0,
                CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
                MiscFlags: 0,
            };
            let mut t = None;
            unsafe {
                self.dev
                    .CreateTexture2D(&desc, None, Some(&mut t))
                    .map_err(|e| format!("staging CreateTexture2D({format:?}) failed: {e}"))?;
            }
            t.ok_or_else(|| "no staging texture".to_owned())
        }
    }
    impl Drop for ZeroCopy {
        fn drop(&mut self) {
            unsafe {
                (self.wgl.unregister)(self.wgl.interop, self.wgl.obj);
                (self.wgl.close)(self.wgl.interop);
            }
        }
    }
}


#[cfg(target_os = "windows")]
pub fn probe_dx_interop(codec: &str) {
    const WIDTH: u32 = 320;
    const HEIGHT: u32 = 240;
    match zerocopy::ZeroCopy::new(codec, WIDTH, HEIGHT, 30, 0, 1, 20_000_000) {
        Ok(mut zc) => {
            let s1 = match zc.selftest_rgba() {
                Ok(px) => format!("RGBA={px:?}"),
                Err(e) => format!("FAILED - {e}"),
            };
            let s2 = match zc.selftest_nv12() {
                Ok(v) => format!("YUV={v:?} (expect [106, 202, 222])"),
                Err(e) => format!("FAILED - {e}"),
            };
            let s3 = match zc.submit(0, true).and_then(|()| zc.retrieve(0)) {
                Ok(d) => format!("OK {} bytes", d.len()),
                Err(e) => format!("FAILED - {e}"),
            };
            info!("DX interop: S1 {} | S2 {} | NVENC {}", s1, s2, s3);
        }
        Err(e) => warn!("DX interop: FAILED - {e}"),
    }
}

#[cfg(not(target_os = "windows"))]
pub fn probe_dx_interop(_codec: &str) {}
#[cfg(not(target_os = "windows"))]
pub mod zerocopy {
    use super::*;

    pub struct ZeroCopy {
        pub frames: u64,
        pub bytes: u64,
    }

    impl ZeroCopy {
        pub fn new(
            _codec: &str,
            _w: u32,
            _h: u32,
            _fps: u32,
            _preset_rank: u32,
            _rc_mode: u32,
            _rc_value: u32,
        ) -> Result<Self, String> {
            Err("zero-copy is windows-only".to_owned())
        }
        pub fn ring() -> usize {
            1
        }
        pub fn lock(&self, _slot: usize) -> Result<(), String> {
            Ok(())
        }
        pub fn unlock(&self, _slot: usize) {}
        pub fn blit_from(&self, _slot: usize, _src: &RenderTarget) {}
        pub fn submit(&mut self, _slot: usize, _force_idr: bool) -> Result<(), String> {
            Err("zero-copy is windows-only".to_owned())
        }
        pub fn retrieve(&mut self, _slot: usize) -> Result<Vec<u8>, String> {
            Err("zero-copy is windows-only".to_owned())
        }
    }
}



pub fn mux_raw_video(
    ffmpeg: &Path,
    raw_video: &Path,
    audio: &Path,
    output: &Path,
    audio_codec: &str,
    strict_flag: &str,
    container: &str,
) -> Result<std::process::Child> {
    let mut cmd = cmd_hidden(ffmpeg);
    cmd.arg("-y")
        .arg("-i")
        .arg(raw_video)
        .arg("-i")
        .arg(audio)
        .args(["-c:v", "copy", "-c:a", audio_codec])
        .args(["-map", "0:v:0", "-map", "1:a:0", "-shortest"]);
    if !strict_flag.is_empty() {
        cmd.args(strict_flag.split_whitespace());
    }
    cmd.args(["-f", container])
        .arg(output)
        .args(["-loglevel", "error"])
        .stdin(Stdio::null())
        .stderr(Stdio::piped());
    cmd.spawn().context("failed to spawn ffmpeg for zero-copy mux")
}

