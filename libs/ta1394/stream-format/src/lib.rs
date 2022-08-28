// SPDX-License-Identifier: MIT
// Copyright (c) 2022 Takashi Sakamoto

#![doc = include_str!("../README.md")]

use ta1394_avc_general::*;

/// The attribute for multi bit audio data in AM824 format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Am824MultiBitAudioAttr {
    pub freq: u32,
    pub rate_ctl: bool,
}

impl Default for Am824MultiBitAudioAttr {
    fn default() -> Self {
        Self {
            freq: 22050,
            rate_ctl: Default::default(),
        }
    }
}

impl Am824MultiBitAudioAttr {
    const FREQ_CODE_22050: u8 = 0x00;
    const FREQ_CODE_24000: u8 = 0x01;
    const FREQ_CODE_32000: u8 = 0x02;
    const FREQ_CODE_44100: u8 = 0x03;
    const FREQ_CODE_48000: u8 = 0x04;
    const FREQ_CODE_96000: u8 = 0x05;
    const FREQ_CODE_176400: u8 = 0x06;
    const FREQ_CODE_192000: u8 = 0x07;

    const FREQ_CODE_MASK: u8 = 0x0f;
    const FREQ_CODE_SHIFT: usize = 4;

    const RATE_CTL_SUPPORTED: u8 = 0x00;
    const RATE_CTL_DONT_CARE: u8 = 0x01;

    const RATE_CTL_MASK: u8 = 0x01;
    const RATE_CTL_SHIFT: usize = 0;

    const LENGTH: usize = 2;

    fn from_raw(raw: &[u8]) -> Self {
        assert!(raw.len() >= Self::LENGTH);
        let freq_code = (raw[0] >> Self::FREQ_CODE_SHIFT) & Self::FREQ_CODE_MASK;
        let freq = match freq_code {
            Self::FREQ_CODE_22050 => 22050,
            Self::FREQ_CODE_24000 => 24000,
            Self::FREQ_CODE_32000 => 32000,
            Self::FREQ_CODE_44100 => 44100,
            Self::FREQ_CODE_48000 => 48000,
            Self::FREQ_CODE_96000 => 96000,
            Self::FREQ_CODE_176400 => 176400,
            Self::FREQ_CODE_192000 => 192000,
            _ => 0xffffffff,
        };

        let rate_ctl_code = (raw[0] >> Self::RATE_CTL_SHIFT) & Self::RATE_CTL_MASK;
        let rate_ctl = rate_ctl_code == Self::RATE_CTL_SUPPORTED;

        Am824MultiBitAudioAttr { freq, rate_ctl }
    }

    fn to_raw(&self) -> [u8; Self::LENGTH] {
        let freq_code = match self.freq {
            22050 => Self::FREQ_CODE_22050,
            24000 => Self::FREQ_CODE_24000,
            32000 => Self::FREQ_CODE_32000,
            44100 => Self::FREQ_CODE_44100,
            48000 => Self::FREQ_CODE_48000,
            96000 => Self::FREQ_CODE_96000,
            176400 => Self::FREQ_CODE_176400,
            192000 => Self::FREQ_CODE_192000,
            _ => 0x0f,
        };

        let rate_ctl_code = if self.rate_ctl {
            Self::RATE_CTL_SUPPORTED
        } else {
            Self::RATE_CTL_DONT_CARE
        };

        let mut raw = [0xff; Self::LENGTH];
        raw[0] = ((freq_code & Self::FREQ_CODE_MASK) << Self::FREQ_CODE_SHIFT)
            | ((rate_ctl_code & Self::RATE_CTL_MASK) << Self::RATE_CTL_SHIFT);
        raw
    }
}

/// The attribute for one bit audio data in AM824 format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Am824OneBitAudioAttr {
    pub freq: u32,
    pub rate_ctl: bool,
}

impl Default for Am824OneBitAudioAttr {
    fn default() -> Self {
        Self {
            freq: 2048000,
            rate_ctl: Default::default(),
        }
    }
}

impl Am824OneBitAudioAttr {
    const FREQ_CODE_2048000: u8 = 0x00;
    const FREQ_CODE_2822400: u8 = 0x01;
    const FREQ_CODE_3072000: u8 = 0x02;
    const FREQ_CODE_5644800: u8 = 0x03;
    const FREQ_CODE_6144000: u8 = 0x04;
    const FREQ_CODE_11289600: u8 = 0x05;
    const FREQ_CODE_12288000: u8 = 0x06;

    const FREQ_CODE_MASK: u8 = 0x0f;
    const FREQ_CODE_SHIFT: usize = 4;

    const RATE_CTL_SUPPORTED: u8 = 0x00;
    const RATE_CTL_DONT_CARE: u8 = 0x01;

    const RATE_CTL_MASK: u8 = 0x01;
    const RATE_CTL_SHIFT: usize = 0;

    const LENGTH: usize = 2;

    fn from_raw(raw: &[u8]) -> Self {
        assert!(raw.len() >= Self::LENGTH);
        let freq_code = (raw[0] >> Self::FREQ_CODE_SHIFT) & Self::FREQ_CODE_MASK;
        let freq = match freq_code {
            Self::FREQ_CODE_2048000 => 2048000,
            Self::FREQ_CODE_2822400 => 2822400,
            Self::FREQ_CODE_3072000 => 3072000,
            Self::FREQ_CODE_5644800 => 5644800,
            Self::FREQ_CODE_6144000 => 6144000,
            Self::FREQ_CODE_11289600 => 11289600,
            Self::FREQ_CODE_12288000 => 12288000,
            _ => 0xffffffff,
        };

        let rate_ctl_code = (raw[0] >> Self::RATE_CTL_SHIFT) & Self::RATE_CTL_MASK;
        let rate_ctl = rate_ctl_code == Self::RATE_CTL_SUPPORTED;

        Am824OneBitAudioAttr { freq, rate_ctl }
    }

    fn to_raw(&self) -> [u8; Self::LENGTH] {
        let freq_code = match self.freq {
            2048000 => Self::FREQ_CODE_2048000,
            2822400 => Self::FREQ_CODE_2822400,
            3072000 => Self::FREQ_CODE_3072000,
            5644800 => Self::FREQ_CODE_5644800,
            6144000 => Self::FREQ_CODE_6144000,
            11289600 => Self::FREQ_CODE_11289600,
            12288000 => Self::FREQ_CODE_12288000,
            _ => 0x0f,
        };

        let rate_ctl_code = if self.rate_ctl {
            Self::RATE_CTL_SUPPORTED
        } else {
            Self::RATE_CTL_DONT_CARE
        };

        let mut raw = [0xff; Self::LENGTH];
        raw[0] = ((freq_code & Self::FREQ_CODE_MASK) << Self::FREQ_CODE_SHIFT)
            | ((rate_ctl_code & Self::RATE_CTL_MASK) << Self::RATE_CTL_SHIFT);
        raw
    }
}

/// The stream type for AM824 format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Am824Stream {
    Iec60958_3(Am824MultiBitAudioAttr),
    Iec61937_3(Am824MultiBitAudioAttr),
    Iec61937_4(Am824MultiBitAudioAttr),
    Iec61937_5(Am824MultiBitAudioAttr),
    Iec61937_6(Am824MultiBitAudioAttr),
    Iec61937_7(Am824MultiBitAudioAttr),
    MultiBitLinearAudioRaw(Am824MultiBitAudioAttr),
    MultiBitLinearAudioDvd(Am824MultiBitAudioAttr),
    OneBitAudioPlainRaw(Am824OneBitAudioAttr),
    OneBitAudioPlainSacd(Am824OneBitAudioAttr),
    OneBitAudioEncodedRaw(Am824OneBitAudioAttr),
    OneBitAudioEncodedSacd(Am824OneBitAudioAttr),
    HighPrecisionMultiBitLinearAudio(Am824MultiBitAudioAttr),
    MidiConformant([u8; 2]),
    Reserved([u8; 4]),
}

impl Default for Am824Stream {
    fn default() -> Self {
        Self::Reserved([0xff; 4])
    }
}

impl Am824Stream {
    const IEC60958_3: u8 = 0x00;
    const IEC61937_3: u8 = 0x01;
    const IEC61937_4: u8 = 0x02;
    const IEC61937_5: u8 = 0x03;
    const IEC61937_6: u8 = 0x04;
    const IEC61937_7: u8 = 0x05;
    const MULTI_BIT_LINEAR_AUDIO_RAW: u8 = 0x06;
    const MULTI_BIT_LINEAR_AUDIO_DVD: u8 = 0x07;
    const ONE_BIT_AUDIO_PLAIN_RAW: u8 = 0x08;
    const ONE_BIT_AUDIO_PLAIN_SACD: u8 = 0x09;
    const ONE_BIT_AUDIO_ENCODED_RAW: u8 = 0x0a;
    const ONE_BIT_AUDIO_ENCODED_SACD: u8 = 0x0b;
    const HIGH_PRECISION_MULTI_BIT_LINEAR_AUDIO: u8 = 0x0c;
    const MIDI_CONFORMANT: u8 = 0x0d;

    const LENGTH: usize = 4;

    fn from_raw(raw: &[u8]) -> Self {
        assert!(raw.len() >= Self::LENGTH);
        match raw[0] {
            Self::IEC60958_3
            | Self::IEC61937_3
            | Self::IEC61937_4
            | Self::IEC61937_5
            | Self::IEC61937_6
            | Self::IEC61937_7
            | Self::MULTI_BIT_LINEAR_AUDIO_RAW
            | Self::MULTI_BIT_LINEAR_AUDIO_DVD
            | Self::HIGH_PRECISION_MULTI_BIT_LINEAR_AUDIO => {
                let attrs = Am824MultiBitAudioAttr::from_raw(&raw[2..]);
                match raw[0] {
                    Self::IEC60958_3 => Self::Iec60958_3(attrs),
                    Self::IEC61937_3 => Self::Iec61937_3(attrs),
                    Self::IEC61937_4 => Self::Iec61937_4(attrs),
                    Self::IEC61937_5 => Self::Iec61937_5(attrs),
                    Self::IEC61937_6 => Self::Iec61937_6(attrs),
                    Self::IEC61937_7 => Self::Iec61937_7(attrs),
                    Self::MULTI_BIT_LINEAR_AUDIO_RAW => Self::MultiBitLinearAudioRaw(attrs),
                    Self::MULTI_BIT_LINEAR_AUDIO_DVD => Self::MultiBitLinearAudioDvd(attrs),
                    Self::HIGH_PRECISION_MULTI_BIT_LINEAR_AUDIO => {
                        Self::HighPrecisionMultiBitLinearAudio(attrs)
                    }
                    _ => unreachable!(),
                }
            }
            Self::ONE_BIT_AUDIO_PLAIN_RAW
            | Self::ONE_BIT_AUDIO_PLAIN_SACD
            | Self::ONE_BIT_AUDIO_ENCODED_RAW
            | Self::ONE_BIT_AUDIO_ENCODED_SACD => {
                let attrs = Am824OneBitAudioAttr::from_raw(&raw[2..]);
                match raw[0] {
                    Self::ONE_BIT_AUDIO_PLAIN_RAW => Self::OneBitAudioPlainRaw(attrs),
                    Self::ONE_BIT_AUDIO_PLAIN_SACD => Self::OneBitAudioPlainSacd(attrs),
                    Self::ONE_BIT_AUDIO_ENCODED_RAW => Self::OneBitAudioEncodedRaw(attrs),
                    Self::ONE_BIT_AUDIO_ENCODED_SACD => Self::OneBitAudioEncodedSacd(attrs),
                    _ => unreachable!(),
                }
            }
            Self::MIDI_CONFORMANT => {
                let mut r = [0; 2];
                r.copy_from_slice(&raw[2..4]);
                Self::MidiConformant(r)
            }
            _ => {
                let mut r = [0xff; 4];
                r.copy_from_slice(&raw);
                Self::Reserved(r)
            }
        }
    }

    fn to_raw(&self) -> [u8; Self::LENGTH] {
        let mut raw = [0xff; Self::LENGTH];
        match self {
            Self::Iec60958_3(attrs)
            | Self::Iec61937_3(attrs)
            | Self::Iec61937_4(attrs)
            | Self::Iec61937_5(attrs)
            | Self::Iec61937_6(attrs)
            | Self::Iec61937_7(attrs)
            | Self::MultiBitLinearAudioRaw(attrs)
            | Self::MultiBitLinearAudioDvd(attrs)
            | Self::HighPrecisionMultiBitLinearAudio(attrs) => {
                raw[0] = match self {
                    Self::Iec60958_3(_) => Self::IEC60958_3,
                    Self::Iec61937_3(_) => Self::IEC61937_3,
                    Self::Iec61937_4(_) => Self::IEC61937_4,
                    Self::Iec61937_5(_) => Self::IEC61937_5,
                    Self::Iec61937_6(_) => Self::IEC61937_6,
                    Self::Iec61937_7(_) => Self::IEC61937_7,
                    Self::MultiBitLinearAudioRaw(_) => Self::MULTI_BIT_LINEAR_AUDIO_RAW,
                    Self::MultiBitLinearAudioDvd(_) => Self::MULTI_BIT_LINEAR_AUDIO_DVD,
                    Self::HighPrecisionMultiBitLinearAudio(_) => {
                        Self::HIGH_PRECISION_MULTI_BIT_LINEAR_AUDIO
                    }
                    _ => unreachable!(),
                };
                raw[2..4].copy_from_slice(&attrs.to_raw());
                raw
            }
            Self::OneBitAudioPlainRaw(attrs)
            | Self::OneBitAudioPlainSacd(attrs)
            | Self::OneBitAudioEncodedRaw(attrs)
            | Self::OneBitAudioEncodedSacd(attrs) => {
                raw[0] = match self {
                    Self::OneBitAudioPlainRaw(_) => Self::ONE_BIT_AUDIO_PLAIN_RAW,
                    Self::OneBitAudioPlainSacd(_) => Self::ONE_BIT_AUDIO_PLAIN_SACD,
                    Self::OneBitAudioEncodedRaw(_) => Self::ONE_BIT_AUDIO_ENCODED_RAW,
                    Self::OneBitAudioEncodedSacd(_) => Self::ONE_BIT_AUDIO_ENCODED_SACD,
                    _ => unreachable!(),
                };
                raw[2..4].copy_from_slice(&attrs.to_raw());
                raw
            }
            Self::MidiConformant(d) => {
                raw[0] = Self::MIDI_CONFORMANT;
                raw[2..4].copy_from_slice(d);
                raw
            }
            Self::Reserved(r) => *r,
        }
    }
}

/// The stream type for compound AM824 format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompoundAm824StreamFormat {
    /// S/PDIF for either uncompressed (PCM) or compressed (AC3/WMA) stream.
    Iec60958_3,
    /// AC 3 compressed stream.
    Iec61937_3,
    /// MPEG compressed stream.
    Iec61937_4,
    /// DTS (Digital Theater Systems) compressed stream.
    Iec61937_5,
    /// MPEG-2 AAC compressed stream.
    Iec61937_6,
    /// ATRAC and ATRAC 2/3 compressed stream.
    Iec61937_7,
    /// Uncompressed linear PCM data stream.
    MultiBitLinearAudioRaw,
    MultiBitLinearAudioDvd,
    HighPrecisionMultiBitLinearAudio,
    /// Multiplexed MIDI stream.
    MidiConformant,
    /// SMPTE time code defined in Audio and Music Data Transmission Protocol v2.1.
    SmpteTimeCodeConformant,
    /// Sample count defined in Audio and Music Data Transmission Protocol v2.1.
    SampleCount,
    /// Ancillary data defined in Audio and Music Data Transmission Protocol v2.1.
    AncillaryData,
    /// Delivery of synchronization information.
    SyncStream,
    Reserved(u8),
}

impl Default for CompoundAm824StreamFormat {
    fn default() -> Self {
        Self::Reserved(0xff)
    }
}

impl CompoundAm824StreamFormat {
    const IEC60958_3: u8 = 0x00;
    const IEC61937_3: u8 = 0x01;
    const IEC61937_4: u8 = 0x02;
    const IEC61937_5: u8 = 0x03;
    const IEC61937_6: u8 = 0x04;
    const IEC61937_7: u8 = 0x05;
    const MULTI_BIT_LINEAR_AUDIO_RAW: u8 = 0x06;
    const MULTI_BIT_LINEAR_AUDIO_DVD: u8 = 0x07;
    const HIGH_PRECISION_MULTI_BIT_LINEAR_AUDIO: u8 = 0x0c;
    const MIDI_CONFORMANT: u8 = 0x0d;
    const SMPTE_TIME_CODE_CONFORMANT: u8 = 0x0e;
    const SAMPLE_COUNT: u8 = 0x0f;
    const ANCILLARY_DATA: u8 = 0x10;
    const SYNC_STREAM: u8 = 0x40;

    fn from_val(val: u8) -> Self {
        match val {
            Self::IEC60958_3 => Self::Iec60958_3,
            Self::IEC61937_3 => Self::Iec61937_3,
            Self::IEC61937_4 => Self::Iec61937_4,
            Self::IEC61937_5 => Self::Iec61937_5,
            Self::IEC61937_6 => Self::Iec61937_6,
            Self::IEC61937_7 => Self::Iec61937_7,
            Self::MULTI_BIT_LINEAR_AUDIO_RAW => Self::MultiBitLinearAudioRaw,
            Self::MULTI_BIT_LINEAR_AUDIO_DVD => Self::MultiBitLinearAudioDvd,
            Self::HIGH_PRECISION_MULTI_BIT_LINEAR_AUDIO => Self::HighPrecisionMultiBitLinearAudio,
            Self::MIDI_CONFORMANT => Self::MidiConformant,
            Self::SMPTE_TIME_CODE_CONFORMANT => Self::SmpteTimeCodeConformant,
            Self::SAMPLE_COUNT => Self::SampleCount,
            Self::ANCILLARY_DATA => Self::AncillaryData,
            Self::SYNC_STREAM => Self::SyncStream,
            _ => Self::Reserved(val),
        }
    }

    fn to_val(&self) -> u8 {
        match self {
            Self::Iec60958_3 => Self::IEC60958_3,
            Self::Iec61937_3 => Self::IEC61937_3,
            Self::Iec61937_4 => Self::IEC61937_4,
            Self::Iec61937_5 => Self::IEC61937_5,
            Self::Iec61937_6 => Self::IEC61937_6,
            Self::Iec61937_7 => Self::IEC61937_7,
            Self::MultiBitLinearAudioRaw => Self::MULTI_BIT_LINEAR_AUDIO_RAW,
            Self::MultiBitLinearAudioDvd => Self::MULTI_BIT_LINEAR_AUDIO_DVD,
            Self::HighPrecisionMultiBitLinearAudio => Self::HIGH_PRECISION_MULTI_BIT_LINEAR_AUDIO,
            Self::MidiConformant => Self::MIDI_CONFORMANT,
            Self::SmpteTimeCodeConformant => Self::SMPTE_TIME_CODE_CONFORMANT,
            Self::SampleCount => Self::SAMPLE_COUNT,
            Self::AncillaryData => Self::ANCILLARY_DATA,
            Self::SyncStream => Self::SYNC_STREAM,
            Self::Reserved(val) => *val,
        }
    }
}

/// The entry of stream format.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompoundAm824StreamEntry {
    /// The number of stream formats.
    pub count: u8,
    /// The stream format.
    pub format: CompoundAm824StreamFormat,
}

impl Default for CompoundAm824StreamEntry {
    fn default() -> Self {
        Self {
            count: 0,
            format: Default::default(),
        }
    }
}

impl CompoundAm824StreamEntry {
    const LENGTH: usize = 2;

    fn from_raw(raw: &[u8]) -> Self {
        assert!(raw.len() >= Self::LENGTH);
        Self {
            count: raw[0],
            format: CompoundAm824StreamFormat::from_val(raw[1]),
        }
    }

    fn to_raw(&self) -> [u8; Self::LENGTH] {
        [self.count, self.format.to_val()]
    }
}

/// Whether to support command-based rate control.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RateCtl {
    Supported,
    DontCare,
    NotSupported,
    Reserved(u8),
}

impl Default for RateCtl {
    fn default() -> Self {
        Self::Reserved(0xff)
    }
}

impl RateCtl {
    const SUPPORTED: u8 = 0x00;
    const DONT_CARE: u8 = 0x01;
    const NOT_SUPPORTED: u8 = 0x02;

    fn to_val(&self) -> u8 {
        match self {
            Self::Supported => Self::SUPPORTED,
            Self::DontCare => Self::DONT_CARE,
            Self::NotSupported => Self::NOT_SUPPORTED,
            Self::Reserved(val) => *val,
        }
    }

    fn from_val(val: u8) -> Self {
        match val {
            Self::SUPPORTED => Self::Supported,
            Self::DONT_CARE => Self::DontCare,
            Self::NOT_SUPPORTED => Self::NotSupported,
            _ => Self::Reserved(val),
        }
    }
}

/// The stream format of compound AM824.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompoundAm824Stream {
    /// The nominal sampling frequency.
    pub freq: u32,
    /// The synchronization source.
    pub sync_src: bool,
    /// Whether to support command-based rate control.
    pub rate_ctl: RateCtl,
    /// The entries of available stream format.
    pub entries: Vec<CompoundAm824StreamEntry>,
}

impl Default for CompoundAm824Stream {
    fn default() -> Self {
        Self {
            freq: 22050,
            sync_src: Default::default(),
            rate_ctl: Default::default(),
            entries: Default::default(),
        }
    }
}

impl CompoundAm824Stream {
    const FREQ_CODE_22050: u8 = 0x00;
    const FREQ_CODE_24000: u8 = 0x01;
    const FREQ_CODE_32000: u8 = 0x02;
    const FREQ_CODE_44100: u8 = 0x03;
    const FREQ_CODE_48000: u8 = 0x04;
    const FREQ_CODE_96000: u8 = 0x05;
    const FREQ_CODE_176400: u8 = 0x06;
    const FREQ_CODE_192000: u8 = 0x07;
    const FREQ_CODE_88200: u8 = 0x0a;

    const SYNC_SRC_MASK: u8 = 0x01;
    const SYNC_SRC_SHIFT: usize = 2;

    const RATE_CTL_MASK: u8 = 0x03;
    const RATE_CTL_SHIFT: usize = 0;

    const LENGTH_MIN: usize = 3;

    fn from_raw(raw: &[u8]) -> Self {
        assert!(raw.len() >= Self::LENGTH_MIN);
        let freq = match raw[0] {
            Self::FREQ_CODE_22050 => 22050,
            Self::FREQ_CODE_24000 => 24000,
            Self::FREQ_CODE_32000 => 32000,
            Self::FREQ_CODE_44100 => 44100,
            Self::FREQ_CODE_48000 => 48000,
            Self::FREQ_CODE_96000 => 96000,
            Self::FREQ_CODE_176400 => 176400,
            Self::FREQ_CODE_192000 => 192000,
            Self::FREQ_CODE_88200 => 88200,
            _ => u32::MAX,
        };
        let sync_src_code = (raw[1] >> Self::SYNC_SRC_SHIFT) & Self::SYNC_SRC_MASK;
        let sync_src = sync_src_code > 0;
        let rate_ctl_code = (raw[1] >> Self::RATE_CTL_SHIFT) & Self::RATE_CTL_MASK;
        let rate_ctl = RateCtl::from_val(rate_ctl_code);
        let entry_count = raw[2] as usize;
        let entries = (0..entry_count)
            .filter_map(|i| {
                let pos = 3 + i * 2;
                if pos + CompoundAm824StreamEntry::LENGTH > raw.len() {
                    None
                } else {
                    Some(CompoundAm824StreamEntry::from_raw(&raw[pos..]))
                }
            })
            .collect();
        Self {
            freq,
            sync_src,
            rate_ctl,
            entries,
        }
    }

    fn to_raw(&self) -> Vec<u8> {
        let mut raw = Vec::with_capacity(Self::LENGTH_MIN);
        let freq_code = match self.freq {
            22050 => Self::FREQ_CODE_22050,
            24000 => Self::FREQ_CODE_24000,
            32000 => Self::FREQ_CODE_32000,
            44100 => Self::FREQ_CODE_44100,
            48000 => Self::FREQ_CODE_48000,
            96000 => Self::FREQ_CODE_96000,
            176400 => Self::FREQ_CODE_176400,
            192000 => Self::FREQ_CODE_192000,
            88200 => Self::FREQ_CODE_88200,
            _ => u8::MAX,
        };
        raw.push(freq_code);

        let sync_src_code = ((self.sync_src as u8) & Self::SYNC_SRC_MASK) << Self::SYNC_SRC_SHIFT;
        let rate_ctl_code = (self.rate_ctl.to_val() & Self::RATE_CTL_MASK) << Self::RATE_CTL_SHIFT;
        raw.push(sync_src_code | rate_ctl_code);

        raw.push(self.entries.len() as u8);
        self.entries.iter().for_each(|entry| {
            raw.extend_from_slice(&entry.to_raw());
        });
        raw
    }
}

/// The type of stream format in Audio and Music hierarchy root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AmStream {
    /// AM824 (single stream) data.
    Am824(Am824Stream),
    /// 24 bit * 4 audio pack data.
    AudioPack,
    /// 32 bit floating point data.
    Fp32,
    /// Compound AM824 (multiplexed stream) data.
    CompoundAm824(CompoundAm824Stream),
    Reserved(Vec<u8>),
}

impl Default for AmStream {
    fn default() -> Self {
        Self::Reserved(vec![0xff; Self::LENGTH_MIN])
    }
}

impl AmStream {
    const HIER_LEVEL_1_AM824: u8 = 0x00;
    const HIER_LEVEL_1_AUDIO_PACK: u8 = 0x01;
    const HIER_LEVEL_1_FP32: u8 = 0x02;
    pub const HIER_LEVEL_1_COMPOUND_AM824: u8 = 0x40;

    const LENGTH_MIN: usize = 4;
}

impl From<&[u8]> for AmStream {
    fn from(raw: &[u8]) -> Self {
        assert!(raw.len() >= Self::LENGTH_MIN);
        match raw[0] {
            Self::HIER_LEVEL_1_AM824 => {
                let format = Am824Stream::from_raw(&raw[1..]);
                Self::Am824(format)
            }
            Self::HIER_LEVEL_1_AUDIO_PACK => Self::AudioPack,
            Self::HIER_LEVEL_1_FP32 => Self::Fp32,
            Self::HIER_LEVEL_1_COMPOUND_AM824 => {
                let s = CompoundAm824Stream::from_raw(&raw[1..]);
                Self::CompoundAm824(s)
            }
            _ => Self::Reserved(raw.to_vec()),
        }
    }
}

impl From<&AmStream> for Vec<u8> {
    fn from(data: &AmStream) -> Self {
        let mut raw = Vec::with_capacity(AmStream::LENGTH_MIN);
        match data {
            AmStream::Am824(format) => {
                raw.push(AmStream::HIER_LEVEL_1_AM824);
                raw.extend_from_slice(&format.to_raw());
            }
            AmStream::AudioPack => {
                raw.push(AmStream::HIER_LEVEL_1_AUDIO_PACK);
                raw.extend_from_slice(&[0xff, 0xff, 0xff, 0xff]);
            }
            AmStream::Fp32 => {
                raw.push(AmStream::HIER_LEVEL_1_FP32);
                raw.extend_from_slice(&[0xff, 0xff, 0xff, 0xff]);
            }
            AmStream::CompoundAm824(s) => {
                raw.push(AmStream::HIER_LEVEL_1_COMPOUND_AM824);
                raw.append(&mut s.to_raw());
            }
            AmStream::Reserved(d) => {
                raw.copy_from_slice(d);
            }
        }
        raw
    }
}

/// The format of stream.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamFormat {
    // Dvcr is not supported currently.
    /// Defined in Audio and Music Data Transmission Protocol.
    Am(AmStream),
    Reserved(Vec<u8>),
}

impl Default for StreamFormat {
    fn default() -> Self {
        Self::Reserved(vec![0xff; Self::LENGTH_MIN])
    }
}

impl StreamFormat {
    /// The value in `format_hierarchy_root` field for Audio and Music.
    pub const HIER_ROOT_AM: u8 = 0x90;

    const LENGTH_MIN: usize = 1;

    fn as_am_stream(&self) -> Option<&AmStream> {
        if let StreamFormat::Am(i) = self {
            Some(i)
        } else {
            None
        }
    }

    /// Detect stream format for AM824 in Audio and Music hierarchy root.
    pub fn as_am824_stream(&self) -> Option<&Am824Stream> {
        if let AmStream::Am824(s) = self.as_am_stream()? {
            Some(s)
        } else {
            None
        }
    }

    /// Detect stream format for Compound AM824 in Audio and Music hierarchy root.
    pub fn as_compound_am824_stream(&self) -> Option<&CompoundAm824Stream> {
        if let AmStream::CompoundAm824(s) = self.as_am_stream()? {
            Some(s)
        } else {
            None
        }
    }

    fn from_raw(raw: &[u8]) -> Self {
        assert!(raw.len() >= Self::LENGTH_MIN);
        match raw[0] {
            Self::HIER_ROOT_AM => StreamFormat::Am(AmStream::from(&raw[1..])),
            _ => StreamFormat::Reserved(raw.to_vec()),
        }
    }

    fn to_raw(&self) -> Vec<u8> {
        let mut raw = Vec::with_capacity(Self::LENGTH_MIN);
        match self {
            StreamFormat::Am(i) => {
                raw.push(StreamFormat::HIER_ROOT_AM);
                raw.append(&mut i.into());
            }
            StreamFormat::Reserved(d) => raw.extend_from_slice(d),
        }
        raw
    }
}

/// The type of plug in unit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnitPlugType {
    Pcr,
    External,
    Async,
    Invalid(u8),
}

impl Default for UnitPlugType {
    fn default() -> Self {
        Self::Invalid(0xff)
    }
}

impl UnitPlugType {
    fn from_val(val: u8) -> Self {
        match val {
            0 => Self::Pcr,
            1 => Self::External,
            2 => Self::Async,
            _ => Self::Invalid(val),
        }
    }

    fn to_val(&self) -> u8 {
        match self {
            Self::Pcr => 0,
            Self::External => 1,
            Self::Async => 2,
            Self::Invalid(val) => *val,
        }
    }
}

/// Data of plug in unit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnitPlugData {
    /// The type of unit.
    pub unit_type: UnitPlugType,
    /// The numeric identifier of plug.
    pub plug_id: u8,
}

impl Default for UnitPlugData {
    fn default() -> Self {
        Self {
            unit_type: Default::default(),
            plug_id: 0xff,
        }
    }
}

impl UnitPlugData {
    const LENGTH: usize = 3;

    fn from_raw(raw: &[u8]) -> Self {
        assert!(raw.len() >= Self::LENGTH);
        Self {
            unit_type: UnitPlugType::from_val(raw[0]),
            plug_id: raw[1],
        }
    }

    fn to_raw(&self) -> [u8; Self::LENGTH] {
        [self.unit_type.to_val(), self.plug_id, 0xff]
    }
}

/// Data of plug in subunit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubunitPlugData {
    pub plug_id: u8,
}

impl Default for SubunitPlugData {
    fn default() -> Self {
        Self { plug_id: 0xff }
    }
}

impl SubunitPlugData {
    const LENGTH: usize = 3;

    fn from_raw(raw: &[u8]) -> Self {
        assert!(raw.len() >= Self::LENGTH);
        Self { plug_id: raw[0] }
    }

    fn to_raw(&self) -> [u8; Self::LENGTH] {
        [self.plug_id, 0xff, 0xff]
    }
}

/// Data of plug in function block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FunctionBlockPlugData {
    /// The type of function block.
    pub fb_type: u8,
    /// The numeric identifier of function block.
    pub fb_id: u8,
    /// The numeric identifier of plug.
    pub plug_id: u8,
}

impl Default for FunctionBlockPlugData {
    fn default() -> Self {
        Self {
            fb_type: 0xff,
            fb_id: 0xff,
            plug_id: 0xff,
        }
    }
}

impl FunctionBlockPlugData {
    const LENGTH: usize = 3;

    fn from_raw(raw: &[u8]) -> Self {
        assert!(raw.len() >= Self::LENGTH);
        Self {
            fb_type: raw[0],
            fb_id: raw[1],
            plug_id: raw[2],
        }
    }

    fn to_raw(&self) -> [u8; Self::LENGTH] {
        [self.fb_type, self.fb_id, self.plug_id]
    }
}

/// Mode of addressing to plug.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlugAddrMode {
    Unit(UnitPlugData),
    Subunit(SubunitPlugData),
    FunctionBlock(FunctionBlockPlugData),
    Invalid([u8; 4]),
}

impl Default for PlugAddrMode {
    fn default() -> Self {
        Self::Invalid([0xff; Self::LENGTH])
    }
}

impl PlugAddrMode {
    const LENGTH: usize = 4;

    fn from_raw(raw: &[u8]) -> Self {
        assert!(raw.len() >= Self::LENGTH);
        match raw[0] {
            0 => Self::Unit(UnitPlugData::from_raw(&raw[1..4])),
            1 => Self::Subunit(SubunitPlugData::from_raw(&raw[1..4])),
            2 => Self::FunctionBlock(FunctionBlockPlugData::from_raw(&raw[1..4])),
            _ => {
                let mut r = [0; Self::LENGTH];
                r.copy_from_slice(raw);
                Self::Invalid(r)
            }
        }
    }

    fn to_raw(&self) -> [u8; Self::LENGTH] {
        let mut raw = [0xff; Self::LENGTH];
        match self {
            Self::Unit(data) => {
                raw[0] = 0;
                raw[1..4].copy_from_slice(&data.to_raw());
            }
            Self::Subunit(data) => {
                raw[0] = 1;
                raw[1..4].copy_from_slice(&data.to_raw());
            }
            Self::FunctionBlock(data) => {
                raw[0] = 2;
                raw[1..4].copy_from_slice(&data.to_raw());
            }
            Self::Invalid(data) => raw.copy_from_slice(data),
        }
        raw
    }
}

/// Direction of stream for plug.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlugDirection {
    Input,
    Output,
    Invalid(u8),
}

impl Default for PlugDirection {
    fn default() -> Self {
        Self::Invalid(0xff)
    }
}

impl PlugDirection {
    const INPUT: u8 = 0;
    const OUTPUT: u8 = 1;

    fn from_val(val: u8) -> Self {
        match val {
            Self::INPUT => Self::Input,
            Self::OUTPUT => Self::Output,
            _ => Self::Invalid(val),
        }
    }

    fn to_val(&self) -> u8 {
        match self {
            Self::Input => Self::INPUT,
            Self::Output => Self::OUTPUT,
            Self::Invalid(val) => *val,
        }
    }
}

/// The address of issued plug.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlugAddr {
    /// The direction of stream for the plug.
    pub direction: PlugDirection,
    /// The mode of addressing.
    pub mode: PlugAddrMode,
}

impl Default for PlugAddr {
    fn default() -> Self {
        Self {
            direction: Default::default(),
            mode: Default::default(),
        }
    }
}

impl PlugAddr {
    const LENGTH: usize = 5;

    fn from_raw(raw: &[u8]) -> Self {
        assert!(raw.len() >= Self::LENGTH);
        Self {
            direction: PlugDirection::from_val(raw[0]),
            mode: PlugAddrMode::from_raw(&raw[1..5]),
        }
    }

    fn to_raw(&self) -> [u8; Self::LENGTH] {
        let mut raw = [0xff; Self::LENGTH];
        raw[0] = self.direction.to_val();
        raw[1..5].copy_from_slice(&self.mode.to_raw());
        raw
    }
}

/// The current status of plug (Table 6.17 – support_status field for SINGLE REQUEST subfunction).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SupportStatus {
    /// The format is already set and stream is available.
    Active,
    /// The format is already set but stream is not available.
    Inactive,
    /// The format is not uset yet.
    NoStreamFormat,
    /// For response frame of specific inquiry operation.
    NoInfo,
    Reserved(u8),
}

impl Default for SupportStatus {
    fn default() -> Self {
        SupportStatus::Reserved(0xff)
    }
}

impl SupportStatus {
    const ACTIVE: u8 = 0x00;
    const INACTIVE: u8 = 0x01;
    const NO_STREAM_FORMAT: u8 = 0x02;
    const NO_INFO: u8 = 0xff;

    fn from_val(val: u8) -> Self {
        match val {
            Self::ACTIVE => Self::Active,
            Self::INACTIVE => Self::Inactive,
            Self::NO_STREAM_FORMAT => Self::NoStreamFormat,
            Self::NO_INFO => Self::NoInfo,
            _ => Self::Reserved(val),
        }
    }

    fn to_val(&self) -> u8 {
        match self {
            Self::Active => Self::ACTIVE,
            Self::Inactive => Self::INACTIVE,
            Self::NoStreamFormat => Self::NO_STREAM_FORMAT,
            Self::NoInfo => Self::NO_INFO,
            Self::Reserved(val) => *val,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExtendedStreamFormat {
    subfunc: u8,
    plug_addr: PlugAddr,
    support_status: SupportStatus,
}

impl Default for ExtendedStreamFormat {
    fn default() -> Self {
        Self {
            subfunc: 0xff,
            plug_addr: Default::default(),
            support_status: Default::default(),
        }
    }
}

impl ExtendedStreamFormat {
    const OPCODE: u8 = 0xbf;

    fn new(subfunc: u8, plug_addr: &PlugAddr) -> Self {
        ExtendedStreamFormat {
            subfunc,
            plug_addr: *plug_addr,
            ..Default::default()
        }
    }

    fn build_operands(
        &mut self,
        _: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        operands.push(self.subfunc);
        operands.extend_from_slice(&self.plug_addr.to_raw());
        operands.push(self.support_status.to_val());
        Ok(())
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        if operands.len() < 7 {
            Err(AvcRespParseError::TooShortResp(7))
        } else if operands[0] != self.subfunc {
            Err(AvcRespParseError::UnexpectedOperands(0))
        } else {
            let plug_addr = PlugAddr::from_raw(&operands[1..6]);
            if plug_addr != self.plug_addr {
                Err(AvcRespParseError::UnexpectedOperands(1))
            } else {
                self.support_status = SupportStatus::from_val(operands[6]);
                Ok(())
            }
        }
    }
}

///
/// SINGLE subfunction of AV/C EXTENDED STREAM FORMAT INFORMATION.
///
/// Described in 6.2.3 SINGLE subfunction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtendedStreamFormatSingle {
    /// Current status of addressed plug.
    pub support_status: SupportStatus,
    /// The format of stream.
    pub stream_format: StreamFormat,
    op: ExtendedStreamFormat,
}

impl ExtendedStreamFormatSingle {
    const SUBFUNC: u8 = 0xc0;

    pub fn new(plug_addr: &PlugAddr) -> Self {
        ExtendedStreamFormatSingle {
            support_status: SupportStatus::NoInfo,
            stream_format: StreamFormat::Reserved(Vec::new()),
            op: ExtendedStreamFormat::new(Self::SUBFUNC, plug_addr),
        }
    }
}

impl AvcOp for ExtendedStreamFormatSingle {
    const OPCODE: u8 = ExtendedStreamFormat::OPCODE;
}

impl AvcStatus for ExtendedStreamFormatSingle {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        self.op.support_status = SupportStatus::Reserved(0xff);
        self.op.build_operands(addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        self.op.parse_operands(addr, operands)?;

        self.stream_format = StreamFormat::from_raw(&operands[7..]);

        self.support_status = self.op.support_status;

        Ok(())
    }
}

impl AvcControl for ExtendedStreamFormatSingle {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        self.op.build_operands(addr, operands)?;
        operands.append(&mut self.stream_format.to_raw());
        Ok(())
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        self.op.parse_operands(addr, operands)?;

        self.stream_format = StreamFormat::from_raw(&operands[7..]);

        self.support_status = self.op.support_status;

        Ok(())
    }
}

///
/// LIST subfunction of AV/C EXTENDED STREAM FORMAT INFORMATION.
///
/// Described in 6.2.4 LIST subfunction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtendedStreamFormatList {
    /// Current status of plug.
    pub support_status: SupportStatus,
    /// The index of stream format.
    pub index: u8,
    /// The format of stream.
    pub stream_format: StreamFormat,
    op: ExtendedStreamFormat,
}

impl ExtendedStreamFormatList {
    const SUBFUNC: u8 = 0xc1;

    pub fn new(plug_addr: &PlugAddr, index: u8) -> Self {
        ExtendedStreamFormatList {
            support_status: SupportStatus::Reserved(0xff),
            index,
            stream_format: StreamFormat::Reserved(Vec::new()),
            op: ExtendedStreamFormat::new(Self::SUBFUNC, plug_addr),
        }
    }
}

impl AvcOp for ExtendedStreamFormatList {
    const OPCODE: u8 = ExtendedStreamFormat::OPCODE;
}

impl AvcStatus for ExtendedStreamFormatList {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        self.op.support_status = SupportStatus::Reserved(0xff);
        self.op.build_operands(addr, operands)?;
        operands.push(self.index);
        Ok(())
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        self.op.parse_operands(addr, operands)?;

        if self.index != operands[7] {
            Err(AvcRespParseError::UnexpectedOperands(7))?;
        }

        self.stream_format = StreamFormat::from_raw(&operands[8..]);

        self.support_status = self.op.support_status;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn am824multibitaudioattr_from() {
        let raw = [0x31, 0xff];
        let attr = Am824MultiBitAudioAttr::from_raw(&raw);
        assert_eq!(44100, attr.freq);
        assert_eq!(false, attr.rate_ctl);
        assert_eq!(raw, attr.to_raw());
    }

    #[test]
    fn am824onebitaudioattr_from() {
        let raw = [0x40, 0xff];
        let attr = Am824OneBitAudioAttr::from_raw(&raw);
        assert_eq!(6144000, attr.freq);
        assert_eq!(true, attr.rate_ctl);
        assert_eq!(raw, attr.to_raw());
    }

    #[test]
    fn am824stream_from() {
        let raw = [0x06, 0xff, 0x20, 0xff];
        let format = Am824Stream::from_raw(&raw);
        let attr = Am824MultiBitAudioAttr {
            freq: 32000,
            rate_ctl: true,
        };
        assert_eq!(format, Am824Stream::MultiBitLinearAudioRaw(attr));
        assert_eq!(raw, format.to_raw());
    }

    #[test]
    fn amstream_from() {
        let raw: &[u8] = &[0x00, 0x08, 0xff, 0x40, 0xff];
        let attr = Am824OneBitAudioAttr {
            freq: 6144000,
            rate_ctl: true,
        };
        let format = AmStream::from(raw);
        assert_eq!(
            AmStream::Am824(Am824Stream::OneBitAudioPlainRaw(attr)),
            format
        );
        assert_eq!(raw, Vec::<u8>::from(&format));

        let raw: &[u8] = &[0x01, 0xff, 0xff, 0xff, 0xff];
        let format = AmStream::from(raw);
        assert_eq!(AmStream::AudioPack, format);
        assert_eq!(raw, Vec::<u8>::from(&format));

        let raw: &[u8] = &[0x02, 0xff, 0xff, 0xff, 0xff];
        let format = AmStream::from(raw);
        assert_eq!(AmStream::Fp32, format);
        assert_eq!(raw, Vec::<u8>::from(&format));
    }

    #[test]
    fn streamformat_from() {
        let raw: &[u8] = &[0x90, 0x00, 0x08, 0xff, 0x40, 0xff];
        let format = StreamFormat::from_raw(raw);
        if let StreamFormat::Am(i) = &format {
            if let AmStream::Am824(s) = i {
                if let Am824Stream::OneBitAudioPlainRaw(attr) = s {
                    assert_eq!(6144000, attr.freq);
                    assert_eq!(true, attr.rate_ctl);
                } else {
                    unreachable!();
                }
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }
        assert_eq!(raw, format.to_raw());

        let mut raw = Vec::<u8>::new();
        raw.extend_from_slice(&[0x90, 0x40, 0x04, 0x02, 0x01, 0x1c, 0x02]);
        let stream_format = StreamFormat::from_raw(&raw);
        if let StreamFormat::Am(i) = &stream_format {
            if let AmStream::CompoundAm824(s) = i {
                assert_eq!(48000, s.freq);
                assert_eq!(false, s.sync_src);
                assert_eq!(RateCtl::NotSupported, s.rate_ctl);
                assert_eq!(1, s.entries.len());
                assert_eq!(0x1c, s.entries[0].count);
                assert_eq!(CompoundAm824StreamFormat::Iec61937_4, s.entries[0].format);
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }
        assert_eq!(raw, stream_format.to_raw());
    }

    #[test]
    fn compoundam824streamformat_from() {
        assert_eq!(0x00, CompoundAm824StreamFormat::from_val(0x00).to_val());
        assert_eq!(0x01, CompoundAm824StreamFormat::from_val(0x01).to_val());
        assert_eq!(0x02, CompoundAm824StreamFormat::from_val(0x02).to_val());
        assert_eq!(0x03, CompoundAm824StreamFormat::from_val(0x03).to_val());
        assert_eq!(0x04, CompoundAm824StreamFormat::from_val(0x04).to_val());
        assert_eq!(0x05, CompoundAm824StreamFormat::from_val(0x05).to_val());
        assert_eq!(0x06, CompoundAm824StreamFormat::from_val(0x06).to_val());
        assert_eq!(0x07, CompoundAm824StreamFormat::from_val(0x07).to_val());
        assert_eq!(0x0c, CompoundAm824StreamFormat::from_val(0x0c).to_val());
        assert_eq!(0x0d, CompoundAm824StreamFormat::from_val(0x0d).to_val());
        assert_eq!(0x0e, CompoundAm824StreamFormat::from_val(0x0e).to_val());
        assert_eq!(0x0f, CompoundAm824StreamFormat::from_val(0x0f).to_val());
        assert_eq!(0x10, CompoundAm824StreamFormat::from_val(0x10).to_val());
        assert_eq!(0x40, CompoundAm824StreamFormat::from_val(0x40).to_val());
        assert_eq!(0xff, CompoundAm824StreamFormat::from_val(0xff).to_val());
    }

    #[test]
    fn compoundam824streamentry_from() {
        assert_eq!(
            [0x02, 0x04],
            CompoundAm824StreamEntry::from_raw(&[0x02, 0x04]).to_raw()
        );
        assert_eq!(
            [0x19, 0x03],
            CompoundAm824StreamEntry::from_raw(&[0x19, 0x03]).to_raw()
        );
        assert_eq!(
            [0x37, 0x00],
            CompoundAm824StreamEntry::from_raw(&[0x37, 0x00]).to_raw()
        );
    }

    #[test]
    fn ratectl_from() {
        assert_eq!(0x00, RateCtl::from_val(0x00).to_val());
        assert_eq!(0x01, RateCtl::from_val(0x01).to_val());
        assert_eq!(0x02, RateCtl::from_val(0x02).to_val());
        assert_eq!(0xff, RateCtl::from_val(0xff).to_val());
    }

    #[test]
    fn compoundam824stream_from() {
        let mut raw = Vec::<u8>::new();
        raw.extend_from_slice(&[0x03, 0x02, 0x02, 0xee, 0x03, 0x37, 0x0d]);
        let s = CompoundAm824Stream::from_raw(&raw);
        assert_eq!(44100, s.freq);
        assert_eq!(false, s.sync_src);
        assert_eq!(RateCtl::NotSupported, s.rate_ctl);
        assert_eq!(2, s.entries.len());
        assert_eq!(0xee, s.entries[0].count);
        assert_eq!(CompoundAm824StreamFormat::Iec61937_5, s.entries[0].format);
        assert_eq!(0x37, s.entries[1].count);
        assert_eq!(
            CompoundAm824StreamFormat::MidiConformant,
            s.entries[1].format
        );
        assert_eq!(raw, CompoundAm824Stream::from_raw(&raw).to_raw());
    }

    #[test]
    fn plug_addr_from() {
        // Unit for PCR stream.
        let addr = PlugAddr {
            direction: PlugDirection::Input,
            mode: PlugAddrMode::Unit(UnitPlugData {
                unit_type: UnitPlugType::Pcr,
                plug_id: 0x2,
            }),
        };
        assert_eq!(addr, PlugAddr::from_raw(&addr.to_raw()));

        // Unit for external stream.
        let addr = PlugAddr {
            direction: PlugDirection::Input,
            mode: PlugAddrMode::Unit(UnitPlugData {
                unit_type: UnitPlugType::External,
                plug_id: 0x3,
            }),
        };
        assert_eq!(addr, PlugAddr::from_raw(&addr.to_raw()));

        // Unit for asynchronous stream.
        let addr = PlugAddr {
            direction: PlugDirection::Output,
            mode: PlugAddrMode::Unit(UnitPlugData {
                unit_type: UnitPlugType::Async,
                plug_id: 0x4,
            }),
        };
        assert_eq!(addr, PlugAddr::from_raw(&addr.to_raw()));

        // Subunit.
        let addr = PlugAddr {
            direction: PlugDirection::Output,
            mode: PlugAddrMode::Subunit(SubunitPlugData { plug_id: 0x8 }),
        };
        assert_eq!(addr, PlugAddr::from_raw(&addr.to_raw()));

        // Function block.
        let addr = PlugAddr {
            direction: PlugDirection::Input,
            mode: PlugAddrMode::FunctionBlock(FunctionBlockPlugData {
                fb_type: 0x1f,
                fb_id: 0x07,
                plug_id: 0x29,
            }),
        };
        assert_eq!(addr, PlugAddr::from_raw(&addr.to_raw()));
    }

    #[test]
    fn single_operands() {
        let plug_addr = PlugAddr {
            direction: PlugDirection::Output,
            mode: PlugAddrMode::Unit(UnitPlugData {
                unit_type: UnitPlugType::Pcr,
                plug_id: 0x03,
            }),
        };
        let mut op = ExtendedStreamFormatSingle::new(&plug_addr);
        let mut operands = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut operands).unwrap();
        assert_eq!(&operands, &[0xc0, 0x01, 0x00, 0x00, 0x03, 0xff, 0xff]);

        let operands = [
            0xc0, 0x01, 0x00, 0x00, 0x03, 0xff, 0x01, 0x90, 0x40, 0x04, 0x00, 0x02, 0x02, 0x06,
            0x02, 0x00,
        ];
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.op.plug_addr, plug_addr);
        assert_eq!(op.op.support_status, SupportStatus::Inactive);

        if let StreamFormat::Am(stream_format) = &op.stream_format {
            if let AmStream::CompoundAm824(s) = stream_format {
                assert_eq!(s.freq, 48000);
                assert_eq!(s.sync_src, false);
                assert_eq!(s.rate_ctl, RateCtl::Supported);
                assert_eq!(s.entries.len(), 2);
                assert_eq!(
                    s.entries[0],
                    CompoundAm824StreamEntry {
                        count: 2,
                        format: CompoundAm824StreamFormat::MultiBitLinearAudioRaw
                    }
                );
                assert_eq!(
                    s.entries[1],
                    CompoundAm824StreamEntry {
                        count: 2,
                        format: CompoundAm824StreamFormat::Iec60958_3
                    }
                );
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }

        let mut operands = Vec::new();
        AvcControl::build_operands(&mut op, &AvcAddr::Unit, &mut operands).unwrap();
        assert_eq!(
            &operands,
            &[
                0xc0, 0x01, 0x00, 0x00, 0x03, 0xff, 0x01, 0x90, 0x40, 0x04, 0x00, 0x02, 0x02, 0x06,
                0x02, 0x00
            ]
        );

        let mut op = ExtendedStreamFormatSingle::new(&plug_addr);
        let operands = [
            0xc0, 0x01, 0x00, 0x00, 0x03, 0xff, 0xff, 0x90, 0x40, 0x05, 0x04, 0x02, 0x02, 0x06,
            0x02, 0x00,
        ];
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.op.plug_addr, plug_addr);
        assert_eq!(op.op.support_status, SupportStatus::NoInfo);
        if let StreamFormat::Am(stream_format) = &op.stream_format {
            if let AmStream::CompoundAm824(s) = stream_format {
                assert_eq!(s.freq, 96000);
                assert_eq!(s.sync_src, true);
                assert_eq!(s.rate_ctl, RateCtl::Supported);
                assert_eq!(s.entries.len(), 2);
                assert_eq!(
                    s.entries[0],
                    CompoundAm824StreamEntry {
                        count: 2,
                        format: CompoundAm824StreamFormat::MultiBitLinearAudioRaw
                    }
                );
                assert_eq!(
                    s.entries[1],
                    CompoundAm824StreamEntry {
                        count: 2,
                        format: CompoundAm824StreamFormat::Iec60958_3
                    }
                );
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }
    }

    #[test]
    fn list_operands() {
        let plug_addr = PlugAddr {
            direction: PlugDirection::Output,
            mode: PlugAddrMode::Unit(UnitPlugData {
                unit_type: UnitPlugType::Pcr,
                plug_id: 0x03,
            }),
        };
        let mut op = ExtendedStreamFormatList::new(&plug_addr, 0x31);
        let mut operands = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut operands).unwrap();
        assert_eq!(&operands, &[0xc1, 0x01, 0x00, 0x00, 0x03, 0xff, 0xff, 0x31]);

        let operands = [
            0xc1, 0x01, 0x00, 0x00, 0x03, 0xff, 0x01, 0x31, 0x90, 0x40, 0x04, 0x00, 0x02, 0x02,
            0x06, 0x02, 0x00,
        ];
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.op.plug_addr, plug_addr);
        assert_eq!(op.op.support_status, SupportStatus::Inactive);
        assert_eq!(op.index, 0x31);
        if let StreamFormat::Am(stream_format) = &op.stream_format {
            if let AmStream::CompoundAm824(s) = stream_format {
                assert_eq!(s.freq, 48000);
                assert_eq!(s.sync_src, false);
                assert_eq!(s.rate_ctl, RateCtl::Supported);
                assert_eq!(s.entries.len(), 2);
                assert_eq!(
                    s.entries[0],
                    CompoundAm824StreamEntry {
                        count: 2,
                        format: CompoundAm824StreamFormat::MultiBitLinearAudioRaw
                    }
                );
                assert_eq!(
                    s.entries[1],
                    CompoundAm824StreamEntry {
                        count: 2,
                        format: CompoundAm824StreamFormat::Iec60958_3
                    }
                );
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }
    }
}
