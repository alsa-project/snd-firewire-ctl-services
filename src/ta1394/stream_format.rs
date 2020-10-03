// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use super::{AvcAddr, Ta1394AvcError};
use super::{AvcOp, AvcStatus, AvcControl};

//
// AV/C STREAM FORMAT INFORMATION
//
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Am824MultiBitAudioAttr {
    pub freq: u32,
    pub rate_ctl: bool,
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
}

impl From<&[u8;2]> for Am824MultiBitAudioAttr {
    fn from(raw: &[u8;2]) -> Self {
        let freq_code =
            (raw[0] >> Am824MultiBitAudioAttr::FREQ_CODE_SHIFT) & Am824MultiBitAudioAttr::FREQ_CODE_MASK;
        let freq = match freq_code {
            Am824MultiBitAudioAttr::FREQ_CODE_22050 => 22050,
            Am824MultiBitAudioAttr::FREQ_CODE_24000 => 24000,
            Am824MultiBitAudioAttr::FREQ_CODE_32000 => 32000,
            Am824MultiBitAudioAttr::FREQ_CODE_44100 => 44100,
            Am824MultiBitAudioAttr::FREQ_CODE_48000 => 48000,
            Am824MultiBitAudioAttr::FREQ_CODE_96000 => 96000,
            Am824MultiBitAudioAttr::FREQ_CODE_176400 => 176400,
            Am824MultiBitAudioAttr::FREQ_CODE_192000 => 192000,
            _ => 0xffffffff,
        };

        let rate_ctl_code =
            (raw[0] >> Am824MultiBitAudioAttr::RATE_CTL_SHIFT) & Am824MultiBitAudioAttr::RATE_CTL_MASK;
        let rate_ctl = rate_ctl_code == Am824MultiBitAudioAttr::RATE_CTL_SUPPORTED;

        Am824MultiBitAudioAttr{
            freq,
            rate_ctl,
        }
    }
}

impl From<&Am824MultiBitAudioAttr> for [u8;2] {
    fn from(data: &Am824MultiBitAudioAttr) -> Self {
        let freq_code = match data.freq {
            22050 => Am824MultiBitAudioAttr::FREQ_CODE_22050,
            24000 => Am824MultiBitAudioAttr::FREQ_CODE_24000,
            32000 => Am824MultiBitAudioAttr::FREQ_CODE_32000,
            44100 => Am824MultiBitAudioAttr::FREQ_CODE_44100,
            48000 => Am824MultiBitAudioAttr::FREQ_CODE_48000,
            96000 => Am824MultiBitAudioAttr::FREQ_CODE_96000,
            176400 => Am824MultiBitAudioAttr::FREQ_CODE_176400,
            192000 => Am824MultiBitAudioAttr::FREQ_CODE_192000,
            _ => 0x0f,
        };

        let rate_ctl_code = if data.rate_ctl {
            Am824MultiBitAudioAttr::RATE_CTL_SUPPORTED
        } else {
            Am824MultiBitAudioAttr::RATE_CTL_DONT_CARE
        };

        let mut raw = [0xff;2];
        raw[0] =
            ((freq_code & Am824MultiBitAudioAttr::FREQ_CODE_MASK) << Am824MultiBitAudioAttr::FREQ_CODE_SHIFT) |
            ((rate_ctl_code & Am824MultiBitAudioAttr::RATE_CTL_MASK) << Am824MultiBitAudioAttr::RATE_CTL_SHIFT);
        raw
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Am824OneBitAudioAttr {
    pub freq: u32,
    pub rate_ctl: bool,
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
}

impl From<&[u8;2]> for Am824OneBitAudioAttr {
    fn from(raw: &[u8;2]) -> Self {
        let freq_code =
            (raw[0] >> Am824OneBitAudioAttr::FREQ_CODE_SHIFT) & Am824OneBitAudioAttr::FREQ_CODE_MASK;
        let freq = match freq_code {
            Am824OneBitAudioAttr::FREQ_CODE_2048000 => 2048000,
            Am824OneBitAudioAttr::FREQ_CODE_2822400 => 2822400,
            Am824OneBitAudioAttr::FREQ_CODE_3072000 => 3072000,
            Am824OneBitAudioAttr::FREQ_CODE_5644800 => 5644800,
            Am824OneBitAudioAttr::FREQ_CODE_6144000 => 6144000,
            Am824OneBitAudioAttr::FREQ_CODE_11289600 => 11289600,
            Am824OneBitAudioAttr::FREQ_CODE_12288000 => 12288000,
            _ => 0xffffffff,
        };

        let rate_ctl_code =
            (raw[0] >> Am824OneBitAudioAttr::RATE_CTL_SHIFT) & Am824OneBitAudioAttr::RATE_CTL_MASK;
        let rate_ctl = rate_ctl_code == Am824OneBitAudioAttr::RATE_CTL_SUPPORTED;

        Am824OneBitAudioAttr{
            freq,
            rate_ctl,
        }
    }
}

impl From<&Am824OneBitAudioAttr> for [u8;2] {
    fn from(data: &Am824OneBitAudioAttr) -> Self {
        let freq_code = match data.freq {
             2048000 => Am824OneBitAudioAttr::FREQ_CODE_2048000,
             2822400 => Am824OneBitAudioAttr::FREQ_CODE_2822400,
             3072000 => Am824OneBitAudioAttr::FREQ_CODE_3072000,
             5644800 => Am824OneBitAudioAttr::FREQ_CODE_5644800,
             6144000 => Am824OneBitAudioAttr::FREQ_CODE_6144000,
             11289600 => Am824OneBitAudioAttr::FREQ_CODE_11289600,
             12288000 => Am824OneBitAudioAttr::FREQ_CODE_12288000,
            _ => 0x0f,
        };

        let rate_ctl_code = if data.rate_ctl {
            Am824OneBitAudioAttr::RATE_CTL_SUPPORTED
        } else {
            Am824OneBitAudioAttr::RATE_CTL_DONT_CARE
        };

        let mut raw = [0xff;2];
        raw[0] =
            ((freq_code & Am824OneBitAudioAttr::FREQ_CODE_MASK) << Am824OneBitAudioAttr::FREQ_CODE_SHIFT) |
            ((rate_ctl_code & Am824OneBitAudioAttr::RATE_CTL_MASK) << Am824OneBitAudioAttr::RATE_CTL_SHIFT);
        raw
    }
}

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
    MidiConformant([u8;2]),
    Reserved([u8;4]),
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
}

impl From<&[u8;4]> for Am824Stream {
    fn from(raw: &[u8;4]) -> Self {
        match raw[0] {
            Am824Stream::IEC60958_3 |
            Am824Stream::IEC61937_3 |
            Am824Stream::IEC61937_4 |
            Am824Stream::IEC61937_5 |
            Am824Stream::IEC61937_6 |
            Am824Stream::IEC61937_7 |
            Am824Stream::MULTI_BIT_LINEAR_AUDIO_RAW |
            Am824Stream::MULTI_BIT_LINEAR_AUDIO_DVD |
            Am824Stream::HIGH_PRECISION_MULTI_BIT_LINEAR_AUDIO => {
                let mut r = [0;2];
                r.copy_from_slice(&raw[2..4]);
                let attrs = Am824MultiBitAudioAttr::from(&r);
                match raw[0] {
                    Am824Stream::IEC60958_3 => Am824Stream::Iec60958_3(attrs),
                    Am824Stream::IEC61937_3 => Am824Stream::Iec61937_3(attrs),
                    Am824Stream::IEC61937_4 => Am824Stream::Iec61937_4(attrs),
                    Am824Stream::IEC61937_5 => Am824Stream::Iec61937_5(attrs),
                    Am824Stream::IEC61937_6 => Am824Stream::Iec61937_6(attrs),
                    Am824Stream::IEC61937_7 => Am824Stream::Iec61937_7(attrs),
                    Am824Stream::MULTI_BIT_LINEAR_AUDIO_RAW =>
                        Am824Stream::MultiBitLinearAudioRaw(attrs),
                    Am824Stream::MULTI_BIT_LINEAR_AUDIO_DVD =>
                        Am824Stream::MultiBitLinearAudioDvd(attrs),
                    Am824Stream::HIGH_PRECISION_MULTI_BIT_LINEAR_AUDIO =>
                        Am824Stream::HighPrecisionMultiBitLinearAudio(attrs),
                    _ => unreachable!(),
                }
            }
            Am824Stream::ONE_BIT_AUDIO_PLAIN_RAW |
            Am824Stream::ONE_BIT_AUDIO_PLAIN_SACD |
            Am824Stream::ONE_BIT_AUDIO_ENCODED_RAW |
            Am824Stream::ONE_BIT_AUDIO_ENCODED_SACD => {
                let mut r = [0;2];
                r.copy_from_slice(&raw[2..4]);
                let attrs = Am824OneBitAudioAttr::from(&r);
                match raw[0] {
                    Am824Stream::ONE_BIT_AUDIO_PLAIN_RAW =>
                        Am824Stream::OneBitAudioPlainRaw(attrs),
                    Am824Stream::ONE_BIT_AUDIO_PLAIN_SACD =>
                        Am824Stream::OneBitAudioPlainSacd(attrs),
                    Am824Stream::ONE_BIT_AUDIO_ENCODED_RAW =>
                        Am824Stream::OneBitAudioEncodedRaw(attrs),
                    Am824Stream::ONE_BIT_AUDIO_ENCODED_SACD =>
                        Am824Stream::OneBitAudioEncodedSacd(attrs),
                    _ => unreachable!(),
                }
            }
            Am824Stream::MIDI_CONFORMANT => {
                let mut r = [0;2];
                r.copy_from_slice(&raw[2..4]);
                Am824Stream::MidiConformant(r)
            }
            _ => Am824Stream::Reserved(*raw),
        }
    }
}

impl From<&Am824Stream> for [u8;4] {
    fn from(format: &Am824Stream) -> Self {
        let mut raw = [0xff;4];
        match format {
            Am824Stream::Iec60958_3(attrs) |
            Am824Stream::Iec61937_3(attrs) |
            Am824Stream::Iec61937_4(attrs) |
            Am824Stream::Iec61937_5(attrs) |
            Am824Stream::Iec61937_6(attrs) |
            Am824Stream::Iec61937_7(attrs) |
            Am824Stream::MultiBitLinearAudioRaw(attrs) |
            Am824Stream::MultiBitLinearAudioDvd(attrs) |
            Am824Stream::HighPrecisionMultiBitLinearAudio(attrs) => {
                raw[0] = match format {
                    Am824Stream::Iec60958_3(_) => Am824Stream::IEC60958_3,
                    Am824Stream::Iec61937_3(_) => Am824Stream::IEC61937_3,
                    Am824Stream::Iec61937_4(_) => Am824Stream::IEC61937_4,
                    Am824Stream::Iec61937_5(_) => Am824Stream::IEC61937_5,
                    Am824Stream::Iec61937_6(_) => Am824Stream::IEC61937_6,
                    Am824Stream::Iec61937_7(_) => Am824Stream::IEC61937_7,
                    Am824Stream::MultiBitLinearAudioRaw(_) =>
                        Am824Stream::MULTI_BIT_LINEAR_AUDIO_RAW,
                    Am824Stream::MultiBitLinearAudioDvd(_) =>
                        Am824Stream::MULTI_BIT_LINEAR_AUDIO_DVD,
                    Am824Stream::HighPrecisionMultiBitLinearAudio(_) =>
                        Am824Stream::HIGH_PRECISION_MULTI_BIT_LINEAR_AUDIO,
                    _ => unreachable!(),
                };
                let a = Into::<[u8;2]>::into(attrs);
                raw[2..4].copy_from_slice(&a);
                raw
            }
            Am824Stream::OneBitAudioPlainRaw(attrs) |
            Am824Stream::OneBitAudioPlainSacd(attrs) |
            Am824Stream::OneBitAudioEncodedRaw(attrs) |
            Am824Stream::OneBitAudioEncodedSacd(attrs) => {
                raw[0] = match format {
                    Am824Stream::OneBitAudioPlainRaw(_) => Am824Stream::ONE_BIT_AUDIO_PLAIN_RAW,
                    Am824Stream::OneBitAudioPlainSacd(_) => Am824Stream::ONE_BIT_AUDIO_PLAIN_SACD,
                    Am824Stream::OneBitAudioEncodedRaw(_) => Am824Stream::ONE_BIT_AUDIO_ENCODED_RAW,
                    Am824Stream::OneBitAudioEncodedSacd(_) => Am824Stream::ONE_BIT_AUDIO_ENCODED_SACD,
                    _ => unreachable!(),
                };
                let a = Into::<[u8;2]>::into(attrs);
                raw[2..4].copy_from_slice(&a);
                raw
            }
            Am824Stream::MidiConformant(d) => {
                let mut raw = [0xff;4];
                raw[0] = Am824Stream::MIDI_CONFORMANT;
                raw[2..4].copy_from_slice(d);
                raw
            }
            Am824Stream::Reserved(raw) => *raw,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompoundAm824StreamFormat{
    Iec60958_3,
    Iec61937_3,
    Iec61937_4,
    Iec61937_5,
    Iec61937_6,
    Iec61937_7,
    MultiBitLinearAudioRaw,
    MultiBitLinearAudioDvd,
    HighPrecisionMultiBitLinearAudio,
    MidiConformant,
    SmpteTimeCodeConformant,
    SampleCount,
    AncillaryData,
    SyncStream,
    Reserved(u8),
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
}

impl From<u8> for CompoundAm824StreamFormat {
    fn from(val: u8) -> Self {
        match val {
            CompoundAm824StreamFormat::IEC60958_3 => CompoundAm824StreamFormat::Iec60958_3,
            CompoundAm824StreamFormat::IEC61937_3 => CompoundAm824StreamFormat::Iec61937_3,
            CompoundAm824StreamFormat::IEC61937_4 => CompoundAm824StreamFormat::Iec61937_4,
            CompoundAm824StreamFormat::IEC61937_5 => CompoundAm824StreamFormat::Iec61937_5,
            CompoundAm824StreamFormat::IEC61937_6 => CompoundAm824StreamFormat::Iec61937_6,
            CompoundAm824StreamFormat::IEC61937_7 => CompoundAm824StreamFormat::Iec61937_7,
            CompoundAm824StreamFormat::MULTI_BIT_LINEAR_AUDIO_RAW => CompoundAm824StreamFormat::MultiBitLinearAudioRaw,
            CompoundAm824StreamFormat::MULTI_BIT_LINEAR_AUDIO_DVD => CompoundAm824StreamFormat::MultiBitLinearAudioDvd,
            CompoundAm824StreamFormat::HIGH_PRECISION_MULTI_BIT_LINEAR_AUDIO => CompoundAm824StreamFormat::HighPrecisionMultiBitLinearAudio,
            CompoundAm824StreamFormat::MIDI_CONFORMANT => CompoundAm824StreamFormat::MidiConformant,
            CompoundAm824StreamFormat::SMPTE_TIME_CODE_CONFORMANT => CompoundAm824StreamFormat::SmpteTimeCodeConformant,
            CompoundAm824StreamFormat::SAMPLE_COUNT => CompoundAm824StreamFormat::SampleCount,
            CompoundAm824StreamFormat::ANCILLARY_DATA => CompoundAm824StreamFormat::AncillaryData,
            CompoundAm824StreamFormat::SYNC_STREAM => CompoundAm824StreamFormat::SyncStream,
            _ => CompoundAm824StreamFormat::Reserved(val),
        }
    }
}

impl From<CompoundAm824StreamFormat> for u8 {
    fn from(fmt: CompoundAm824StreamFormat) -> Self {
        match fmt {
            CompoundAm824StreamFormat::Iec60958_3 => CompoundAm824StreamFormat::IEC60958_3,
            CompoundAm824StreamFormat::Iec61937_3 => CompoundAm824StreamFormat::IEC61937_3,
            CompoundAm824StreamFormat::Iec61937_4 => CompoundAm824StreamFormat::IEC61937_4,
            CompoundAm824StreamFormat::Iec61937_5 => CompoundAm824StreamFormat::IEC61937_5,
            CompoundAm824StreamFormat::Iec61937_6 => CompoundAm824StreamFormat::IEC61937_6,
            CompoundAm824StreamFormat::Iec61937_7 => CompoundAm824StreamFormat::IEC61937_7,
            CompoundAm824StreamFormat::MultiBitLinearAudioRaw => CompoundAm824StreamFormat::MULTI_BIT_LINEAR_AUDIO_RAW,
            CompoundAm824StreamFormat::MultiBitLinearAudioDvd => CompoundAm824StreamFormat::MULTI_BIT_LINEAR_AUDIO_DVD,
            CompoundAm824StreamFormat::HighPrecisionMultiBitLinearAudio => CompoundAm824StreamFormat::HIGH_PRECISION_MULTI_BIT_LINEAR_AUDIO,
            CompoundAm824StreamFormat::MidiConformant => CompoundAm824StreamFormat::MIDI_CONFORMANT,
            CompoundAm824StreamFormat::SmpteTimeCodeConformant => CompoundAm824StreamFormat::SMPTE_TIME_CODE_CONFORMANT,
            CompoundAm824StreamFormat::SampleCount => CompoundAm824StreamFormat::SAMPLE_COUNT,
            CompoundAm824StreamFormat::AncillaryData => CompoundAm824StreamFormat::ANCILLARY_DATA,
            CompoundAm824StreamFormat::SyncStream => CompoundAm824StreamFormat::SYNC_STREAM,
            CompoundAm824StreamFormat::Reserved(val) => val,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompoundAm824StreamEntry{
    pub count: u8,
    pub format: CompoundAm824StreamFormat,
}

impl From<&[u8;2]> for CompoundAm824StreamEntry {
    fn from(raw: &[u8;2]) -> Self {
        CompoundAm824StreamEntry{
            count: raw[0],
            format: CompoundAm824StreamFormat::from(raw[1]),
        }
    }
}

impl From<&CompoundAm824StreamEntry> for [u8;2] {
    fn from(data: &CompoundAm824StreamEntry) -> Self {
        [data.count, data.format.into()]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RateCtl {
    Supported,
    DontCare,
    NotSupported,
    Reserved(u8),
}

impl RateCtl {
    const SUPPORTED: u8 = 0x00;
    const DONT_CARE: u8 = 0x01;
    const NOT_SUPPORTED: u8 = 0x02;
}

impl From<RateCtl> for u8 {
    fn from(rate_ctl: RateCtl) -> Self {
        match rate_ctl {
            RateCtl::Supported => RateCtl::SUPPORTED,
            RateCtl::DontCare => RateCtl::DONT_CARE,
            RateCtl::NotSupported => RateCtl::NOT_SUPPORTED,
            RateCtl::Reserved(val) => val,
        }
    }
}

impl From<u8> for RateCtl {
    fn from(val: u8) -> Self {
        match val {
            Self::SUPPORTED => Self::Supported,
            Self::DONT_CARE => Self::DontCare,
            Self::NOT_SUPPORTED => Self::NotSupported,
            _ => RateCtl::Reserved(val),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompoundAm824Stream{
    pub freq: u32,
    pub sync_src: bool,
    pub rate_ctl: RateCtl,
    pub entries: Vec<CompoundAm824StreamEntry>,
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
}

impl From<&[u8]> for CompoundAm824Stream {
    fn from(raw: &[u8]) -> Self {
        let freq = match raw[0] {
            CompoundAm824Stream::FREQ_CODE_22050 => 22050,
            CompoundAm824Stream::FREQ_CODE_24000 => 24000,
            CompoundAm824Stream::FREQ_CODE_32000 => 32000,
            CompoundAm824Stream::FREQ_CODE_44100 => 44100,
            CompoundAm824Stream::FREQ_CODE_48000 => 48000,
            CompoundAm824Stream::FREQ_CODE_96000 => 96000,
            CompoundAm824Stream::FREQ_CODE_176400 => 176400,
            CompoundAm824Stream::FREQ_CODE_192000 => 192000,
            CompoundAm824Stream::FREQ_CODE_88200 => 88200,
            _ => u32::MAX,
        };
        let sync_src_code =
            (raw[1] >> CompoundAm824Stream::SYNC_SRC_SHIFT) & CompoundAm824Stream::SYNC_SRC_MASK;
        let sync_src = sync_src_code > 0;
        let rate_ctl_code =
            (raw[1] >> CompoundAm824Stream::RATE_CTL_SHIFT) & CompoundAm824Stream::RATE_CTL_MASK;
        let rate_ctl = RateCtl::from(rate_ctl_code);
        let entry_count = raw[2] as usize;
        let entries = (0..entry_count).filter_map(|i| {
            if 3 + i * 2 + 2 > raw.len() {
                None
            } else {
                let mut doublet = [0;2];
                doublet.copy_from_slice(&raw[(3 + i * 2)..(3 + i * 2 + 2)]);
                Some(CompoundAm824StreamEntry::from(&doublet))
            }
        }).collect();
        CompoundAm824Stream{freq, sync_src, rate_ctl, entries}
    }
}

impl From<&CompoundAm824Stream> for Vec<u8> {
    fn from(data: &CompoundAm824Stream) -> Self {
        let mut raw = Vec::new();
        let freq_code = match data.freq {
            22050 => CompoundAm824Stream::FREQ_CODE_22050,
            24000 => CompoundAm824Stream::FREQ_CODE_24000,
            32000 => CompoundAm824Stream::FREQ_CODE_32000,
            44100 => CompoundAm824Stream::FREQ_CODE_44100,
            48000 => CompoundAm824Stream::FREQ_CODE_48000,
            96000 => CompoundAm824Stream::FREQ_CODE_96000,
            176400 => CompoundAm824Stream::FREQ_CODE_176400,
            192000 => CompoundAm824Stream::FREQ_CODE_192000,
            88200 => CompoundAm824Stream::FREQ_CODE_88200,
            _ => u8::MAX,
        };
        raw.push(freq_code);

        let sync_src_code = ((data.sync_src as u8) & CompoundAm824Stream::SYNC_SRC_MASK) <<
                            CompoundAm824Stream::SYNC_SRC_SHIFT;
        let rate_ctl_code = (u8::from(data.rate_ctl) & CompoundAm824Stream::RATE_CTL_MASK) <<
                            CompoundAm824Stream::RATE_CTL_SHIFT;
        raw.push(sync_src_code | rate_ctl_code);

        raw.push(data.entries.len() as u8);
        data.entries.iter().for_each(|entry|{
            raw.extend_from_slice(&Into::<[u8;2]>::into(entry));
        });
        raw
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AmStream{
    Am824(Am824Stream),
    AudioPack,
    Fp32,
    CompoundAm824(CompoundAm824Stream),
    Reserved(Vec<u8>),
}

impl AmStream {
    const HIER_LEVEL_1_AM824: u8 = 0x00;
    const HIER_LEVEL_1_AUDIO_PACK: u8 = 0x01;
    const HIER_LEVEL_1_FP32: u8 = 0x02;
    const HIER_LEVEL_1_COMPOUND_AM824: u8 = 0x40;
}

impl From<&[u8]> for AmStream {
    fn from(raw: &[u8]) -> Self {
        match raw[0] {
            AmStream::HIER_LEVEL_1_AM824 => {
                let mut r = [0xff;4];
                r.copy_from_slice(&raw[1..5]);
                let format = Am824Stream::from(&r);
                AmStream::Am824(format)
            }
            AmStream::HIER_LEVEL_1_AUDIO_PACK => AmStream::AudioPack,
            AmStream::HIER_LEVEL_1_FP32 => AmStream::Fp32,
            AmStream::HIER_LEVEL_1_COMPOUND_AM824 => {
                let s = CompoundAm824Stream::from(&raw[1..]);
                AmStream::CompoundAm824(s)
            }
            _ => AmStream::Reserved((*raw).to_vec()),
        }
    }
}

impl From<&AmStream> for Vec<u8> {
    fn from(data: &AmStream) -> Self {
        let mut raw = Vec::new();
        match data {
            AmStream::Am824(d) => {
                raw.push(AmStream::HIER_LEVEL_1_AM824);
                raw.extend_from_slice(&Into::<[u8;4]>::into(d));
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
                raw.append(&mut Into::<Vec<u8>>::into(s));
            }
            AmStream::Reserved(d) => {
                raw.copy_from_slice(d);
            }
        }
        raw
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamFormat{
    // Dvcr is not supported currently.
    Am(AmStream),
    Reserved(Vec<u8>),
}

impl StreamFormat {
    const HIER_ROOT_AM: u8 = 0x90;

    fn as_am_stream(&self) -> Result<&AmStream, Error> {
        if let StreamFormat::Am(i) = self {
            Ok(i)
        } else {
            let label = "Audio & Music format is not available for the unit";
            Err(Error::new(FileError::Nxio, &label))
        }
    }

    pub fn as_am824_stream(&self) -> Result<&Am824Stream, Error> {
        if let AmStream::Am824(s) = self.as_am_stream()? {
            Ok(s)
        } else {
            let label = "AM824 format is not available for the unit";
            Err(Error::new(FileError::Nxio, &label))
        }
    }

    pub fn as_compound_am824_stream(&self) -> Result<&CompoundAm824Stream, Error> {
        if let AmStream::CompoundAm824(s) = self.as_am_stream()? {
            Ok(s)
        } else {
            let label = "Compound AM824 stream is not available for the unit";
            Err(Error::new(FileError::Nxio, &label))
        }
    }
}

impl From<&[u8]> for StreamFormat {
    fn from(raw: &[u8]) -> Self {
        match raw[0] {
            Self::HIER_ROOT_AM => StreamFormat::Am(AmStream::from(&raw[1..])),
            _ => StreamFormat::Reserved(raw.to_vec()),
        }
    }
}

impl From<&StreamFormat> for Vec<u8> {
    fn from(data: &StreamFormat) -> Self {
        let mut raw = Vec::new();
        match data {
            StreamFormat::Am(i) => {
                raw.push(StreamFormat::HIER_ROOT_AM);
                raw.append(&mut i.into());
            }
            StreamFormat::Reserved(d) => raw.extend_from_slice(d),
        }
        raw
    }
}

//
// AV/C EXTENDED STREAM FORMAT INFORMATION.
//
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnitPlugType {
    Pcr,
    External,
    Async,
    Invalid(u8),
}

impl From<u8> for UnitPlugType {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::Pcr,
            1 => Self::External,
            2 => Self::Async,
            _ => Self::Invalid(val),
        }
    }
}

impl From<UnitPlugType> for u8 {
    fn from(plug_type: UnitPlugType) -> Self {
        match plug_type {
            UnitPlugType::Pcr => 0,
            UnitPlugType::External => 1,
            UnitPlugType::Async => 2,
            UnitPlugType::Invalid(val) => val,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnitPlugData{
    pub unit_type: UnitPlugType,
    pub plug_id: u8,
}

impl From<&[u8]> for UnitPlugData {
    fn from(raw: &[u8]) -> Self {
        UnitPlugData{
            unit_type: raw[0].into(),
            plug_id: raw[1],
        }
    }
}

impl From<UnitPlugData> for [u8;3] {
    fn from(data: UnitPlugData) -> Self {
        [u8::from(data.unit_type), data.plug_id, 0xff]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubunitPlugData{
    pub plug_id: u8,
}

impl From<&[u8]> for SubunitPlugData {
    fn from(raw: &[u8]) -> Self {
        SubunitPlugData{
            plug_id: raw[0],
        }
    }
}

impl From<SubunitPlugData> for [u8;3] {
    fn from(data: SubunitPlugData) -> Self {
        [data.plug_id, 0xff, 0xff]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FunctionBlockPlugData{
    pub fb_type: u8,
    pub fb_id: u8,
    pub plug_id: u8,
}

impl From<&[u8]> for FunctionBlockPlugData {
    fn from(raw: &[u8]) -> Self {
        FunctionBlockPlugData{
            fb_type: raw[0],
            fb_id: raw[1],
            plug_id: raw[2],
        }
    }
}

impl From<FunctionBlockPlugData> for [u8;3] {
    fn from(data: FunctionBlockPlugData) -> Self {
        [data.fb_type, data.fb_id, data.plug_id]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlugAddrMode {
    Unit(UnitPlugData),
    Subunit(SubunitPlugData),
    FunctionBlock(FunctionBlockPlugData),
    Invalid,
}

impl From<&[u8]> for PlugAddrMode {
    fn from(raw: &[u8]) -> Self {
        match raw[0] {
            0 => Self::Unit(UnitPlugData::from(&raw[1..4])),
            1 => Self::Subunit(SubunitPlugData::from(&raw[1..4])),
            2 => Self::FunctionBlock(FunctionBlockPlugData::from(&raw[1..4])),
            _ => Self::Invalid,
        }
    }
}

impl From<PlugAddrMode> for [u8;4] {
    fn from(mode: PlugAddrMode) -> Self {
        let mut raw: [u8;4] = [0xff;4];
        match mode {
            PlugAddrMode::Unit(data) => {
                raw[0] = 0;
                let d: [u8;3] = data.into();
                raw[1..4].copy_from_slice(&d);
            }
            PlugAddrMode::Subunit(data) => {
                raw[0] = 1;
                let d: [u8;3] = data.into();
                raw[1..4].copy_from_slice(&d);
            }
            PlugAddrMode::FunctionBlock(data) => {
                raw[0] = 2;
                let d: [u8;3] = data.into();
                raw[1..4].copy_from_slice(&d);
            }
            _ => (),
        }
        raw
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlugDirection {
    Input,
    Output,
    Invalid(u8),
}

impl PlugDirection {
    const INPUT: u8 = 0;
    const OUTPUT: u8 = 1;
}

impl From<u8> for PlugDirection {
    fn from(val: u8) -> Self {
        match val {
            PlugDirection::INPUT => PlugDirection::Input,
            PlugDirection::OUTPUT => PlugDirection::Output,
            _ => Self::Invalid(val),
        }
    }
}

impl From<PlugDirection> for u8 {
    fn from(dir: PlugDirection) -> Self {
        match dir {
            PlugDirection::Input => PlugDirection::INPUT,
            PlugDirection::Output => PlugDirection::OUTPUT,
            PlugDirection::Invalid(val) => val,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlugAddr{
    pub direction: PlugDirection,
    pub mode: PlugAddrMode,
}

impl From<&[u8]> for PlugAddr {
    fn from(raw: &[u8]) -> Self {
        PlugAddr {
            direction: PlugDirection::from(raw[0]),
            mode: PlugAddrMode::from(&raw[1..5]),
        }
    }
}

impl From<PlugAddr> for [u8;5] {
    fn from(addr: PlugAddr) -> [u8;5] {
        let mut raw = [0xff;5];
        raw[0] = u8::from(addr.direction);
        let m: [u8;4] = addr.mode.into();
        raw[1..5].copy_from_slice(&m);
        raw
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SupportStatus {
    Active,
    Inactive,
    NoStreamFormat,
    NoInfo,
    Reserved(u8),
}

impl SupportStatus {
    const ACTIVE: u8 = 0x00;
    const INACTIVE: u8 = 0x01;
    const NO_STREAM_FORMAT: u8 = 0x02;
    const NO_INFO: u8 = 0xff;
}

impl From<u8> for SupportStatus {
    fn from(val: u8) -> Self {
        match val {
            Self::ACTIVE => Self::Active,
            Self::INACTIVE => Self::Inactive,
            Self::NO_STREAM_FORMAT => Self::NoStreamFormat,
            Self::NO_INFO => Self::NoInfo,
            _ => Self::Reserved(val),
        }
    }
}

impl From<SupportStatus> for u8 {
    fn from(status: SupportStatus) -> Self {
        match status {
            SupportStatus::Active => SupportStatus::ACTIVE,
            SupportStatus::Inactive => SupportStatus::INACTIVE,
            SupportStatus::NoStreamFormat => SupportStatus::NO_STREAM_FORMAT,
            SupportStatus::NoInfo => SupportStatus::NO_INFO,
            SupportStatus::Reserved(val) => val,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExtendedStreamFormat{
    subfunc: u8,
    plug_addr: PlugAddr,
    support_status: SupportStatus,
}

impl ExtendedStreamFormat {
    const OPCODE: u8 = 0xbf;
    
    fn new(subfunc: u8, plug_addr: &PlugAddr) -> Self {
        ExtendedStreamFormat{
            subfunc,
            plug_addr: *plug_addr,
            support_status: SupportStatus::Reserved(0xff),
        }
    }

    fn build_operands(&mut self, _: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        operands.push(self.subfunc);
        operands.extend_from_slice(&Into::<[u8;5]>::into(self.plug_addr));
        operands.push(self.support_status.into());
        Ok(())
    }

    fn parse_operands(&mut self, _: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        if operands.len() < 7 {
            let label = format!("Unexpected length of data for ExtendedStreamFormat: {}",
                                operands.len());
            Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label))
        } else if operands[0] != self.subfunc {
            let label = format!("Unexpected subfunction for ExtendedStreamFormat: {} but {}",
                                self.subfunc, operands[0]);
            Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label))
        } else {
            let plug_addr = PlugAddr::from(&operands[1..6]);
            if plug_addr != self.plug_addr {
                let label = format!("Unexpected address to plug for ExtendedStreamFormat: {:?} but {:?}",
                                    plug_addr, self.plug_addr);
                Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label))
            } else {
                self.support_status = SupportStatus::from(operands[6]);
                Ok(())
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtendedStreamFormatSingle{
    pub support_status: SupportStatus,
    pub stream_format: StreamFormat,
    op: ExtendedStreamFormat,
}

impl ExtendedStreamFormatSingle {
    const SUBFUNC: u8 = 0xc0;

    pub fn new(plug_addr: &PlugAddr) -> Self {
        ExtendedStreamFormatSingle{
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
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        self.op.support_status = SupportStatus::Reserved(0xff);
        self.op.build_operands(addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        self.op.parse_operands(addr, operands)?;

        self.stream_format = StreamFormat::from(&operands[7..]);

        self.support_status = self.op.support_status;

        Ok(())
    }
}

impl AvcControl for ExtendedStreamFormatSingle {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        self.op.build_operands(addr, operands)?;
        operands.append(&mut Into::<Vec<u8>>::into(&self.stream_format));
        Ok(())
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        self.op.parse_operands(addr, operands)?;

        self.stream_format = StreamFormat::from(&operands[7..]);

        self.support_status = self.op.support_status;

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtendedStreamFormatList{
    pub support_status: SupportStatus,
    pub index: u8,
    pub stream_format: StreamFormat,
    op: ExtendedStreamFormat,
}

impl ExtendedStreamFormatList {
    const SUBFUNC: u8 = 0xc1;

    pub fn new(plug_addr: &PlugAddr, index: u8) -> Self {
        ExtendedStreamFormatList{
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
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        self.op.support_status = SupportStatus::Reserved(0xff);
        self.op.build_operands(addr, operands)?;
        operands.push(self.index);
        Ok(())
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        self.op.parse_operands(addr, operands)?;

        if self.index != operands[7] {
            let label = format!("Unexpected value for list index: {:?}", operands[7]);
            return Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label));
        }

        self.stream_format = StreamFormat::from(&operands[8..]);

        self.support_status = self.op.support_status;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Am824MultiBitAudioAttr, Am824OneBitAudioAttr, Am824Stream};
    use super::{CompoundAm824Stream, CompoundAm824StreamEntry, CompoundAm824StreamFormat, RateCtl};
    use super::{AmStream, StreamFormat};

    use super::{UnitPlugType, UnitPlugData, SubunitPlugData, FunctionBlockPlugData, PlugAddrMode, PlugDirection, PlugAddr};

    use super::AvcAddr;
    use super::{AvcStatus, AvcControl};
    use super::{SupportStatus, ExtendedStreamFormatSingle, ExtendedStreamFormatList};

    #[test]
    fn am824multibitaudioattr_from() {
        let raw = [0x31, 0xff];
        let attr = Am824MultiBitAudioAttr::from(&raw);
        assert_eq!(44100, attr.freq);
        assert_eq!(false, attr.rate_ctl);
        assert_eq!(raw, Into::<[u8;2]>::into(&attr));
    }

    #[test]
    fn am824onebitaudioattr_from() {
        let raw = [0x40, 0xff];
        let attr = Am824OneBitAudioAttr::from(&raw);
        assert_eq!(6144000, attr.freq);
        assert_eq!(true, attr.rate_ctl);
        assert_eq!(raw, Into::<[u8;2]>::into(&attr));
    }

    #[test]
    fn am824stream_from() {
        let raw = [0x06, 0xff, 0x20, 0xff];
        let format = Am824Stream::from(&raw);
        let attr = Am824MultiBitAudioAttr{
            freq: 32000,
            rate_ctl: true,
        };
        assert_eq!(format, Am824Stream::MultiBitLinearAudioRaw(attr));
        assert_eq!(raw, Into::<[u8;4]>::into(&format));
    }

    #[test]
    fn amstream_from() {
        let raw: &[u8] = &[0x00, 0x08, 0xff, 0x40, 0xff];
        let attr = Am824OneBitAudioAttr{
            freq: 6144000,
            rate_ctl: true,
        };
        let format = AmStream::from(raw);
        assert_eq!(AmStream::Am824(Am824Stream::OneBitAudioPlainRaw(attr)), format);
        assert_eq!(raw, Into::<Vec<u8>>::into(&format).as_slice());

        let raw: &[u8] = &[0x01, 0xff, 0xff, 0xff, 0xff];
        let format = AmStream::from(raw);
        assert_eq!(AmStream::AudioPack, format);
        assert_eq!(raw, Into::<Vec<u8>>::into(&format).as_slice());

        let raw: &[u8] = &[0x02, 0xff, 0xff, 0xff, 0xff];
        let format = AmStream::from(raw);
        assert_eq!(AmStream::Fp32, format);
        assert_eq!(raw, Into::<Vec<u8>>::into(&format).as_slice());
    }

    #[test]
    fn streamformat_from() {
        let raw: &[u8] = &[0x90, 0x00, 0x08, 0xff, 0x40, 0xff];
        let format = StreamFormat::from(raw);
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
        assert_eq!(raw, Into::<Vec<u8>>::into(&format).as_slice());

        let mut raw = Vec::<u8>::new();
        raw.extend_from_slice(&[0x90, 0x40, 0x04, 0x02, 0x01, 0x1c, 0x02]);
        let stream_format = StreamFormat::from(raw.as_slice());
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
        assert_eq!(raw, Into::<Vec<u8>>::into(&stream_format));
    }

    #[test]
    fn compoundam824streamformat_from() {
        assert_eq!(0x00, u8::from(CompoundAm824StreamFormat::from(0x00)));
        assert_eq!(0x01, u8::from(CompoundAm824StreamFormat::from(0x01)));
        assert_eq!(0x02, u8::from(CompoundAm824StreamFormat::from(0x02)));
        assert_eq!(0x03, u8::from(CompoundAm824StreamFormat::from(0x03)));
        assert_eq!(0x04, u8::from(CompoundAm824StreamFormat::from(0x04)));
        assert_eq!(0x05, u8::from(CompoundAm824StreamFormat::from(0x05)));
        assert_eq!(0x06, u8::from(CompoundAm824StreamFormat::from(0x06)));
        assert_eq!(0x07, u8::from(CompoundAm824StreamFormat::from(0x07)));
        assert_eq!(0x0c, u8::from(CompoundAm824StreamFormat::from(0x0c)));
        assert_eq!(0x0d, u8::from(CompoundAm824StreamFormat::from(0x0d)));
        assert_eq!(0x0e, u8::from(CompoundAm824StreamFormat::from(0x0e)));
        assert_eq!(0x0f, u8::from(CompoundAm824StreamFormat::from(0x0f)));
        assert_eq!(0x10, u8::from(CompoundAm824StreamFormat::from(0x10)));
        assert_eq!(0x40, u8::from(CompoundAm824StreamFormat::from(0x40)));
        assert_eq!(0xff, u8::from(CompoundAm824StreamFormat::from(0xff)));
    }

    #[test]
    fn compoundam824streamentry_from() {
        assert_eq!([0x02, 0x04], Into::<[u8;2]>::into(&CompoundAm824StreamEntry::from(&[0x02, 0x04])));
        assert_eq!([0x19, 0x03], Into::<[u8;2]>::into(&CompoundAm824StreamEntry::from(&[0x19, 0x03])));
        assert_eq!([0x37, 0x00], Into::<[u8;2]>::into(&CompoundAm824StreamEntry::from(&[0x37, 0x00])));
    }

    #[test]
    fn ratectl_from() {
        assert_eq!(0x00, u8::from(RateCtl::from(0x00)));
        assert_eq!(0x01, u8::from(RateCtl::from(0x01)));
        assert_eq!(0x02, u8::from(RateCtl::from(0x02)));
        assert_eq!(0xff, u8::from(RateCtl::from(0xff)));
    }

    #[test]
    fn compoundam824stream_from() {
        let mut raw = Vec::<u8>::new();
        raw.extend_from_slice(&[0x03, 0x02, 0x02, 0xee, 0x03, 0x37, 0x0d]);
        let s = CompoundAm824Stream::from(raw.as_slice());
        assert_eq!(44100, s.freq);
        assert_eq!(false, s.sync_src);
        assert_eq!(RateCtl::NotSupported, s.rate_ctl);
        assert_eq!(2, s.entries.len());
        assert_eq!(0xee, s.entries[0].count);
        assert_eq!(CompoundAm824StreamFormat::Iec61937_5, s.entries[0].format);
        assert_eq!(0x37, s.entries[1].count);
        assert_eq!(CompoundAm824StreamFormat::MidiConformant, s.entries[1].format);
        assert_eq!(raw, Into::<Vec<u8>>::into(&CompoundAm824Stream::from(raw.as_slice())));
    }

    #[test]
    fn plug_addr_from() {
        let unit_pcr = PlugAddr {
            direction: PlugDirection::Input,
            mode: PlugAddrMode::Unit(UnitPlugData {
                unit_type: UnitPlugType::Pcr,
                plug_id: 0x2,
            }),
        };
        let raw: [u8;5] = unit_pcr.into();
        assert_eq!(unit_pcr, raw[..].into());

        let unit_ext = PlugAddr {
            direction: PlugDirection::Input,
            mode: PlugAddrMode::Unit(UnitPlugData {
                unit_type: UnitPlugType::External,
                plug_id: 0x3,
            }),
        };
        let raw: [u8;5] = unit_ext.into();
        assert_eq!(unit_ext, raw[..].into());

        let unit_async = PlugAddr {
            direction: PlugDirection::Output,
            mode: PlugAddrMode::Unit(UnitPlugData {
                unit_type: UnitPlugType::Async,
                plug_id: 0x4,
            }),
        };
        let raw: [u8;5] = unit_async.into();
        assert_eq!(unit_async, raw[..].into());

        let subunit = PlugAddr {
            direction: PlugDirection::Output,
            mode: PlugAddrMode::Subunit(SubunitPlugData { plug_id: 0x8 }),
        };
        let raw: [u8;5] = subunit.into();
        assert_eq!(subunit, raw[..].into());

        let fb = PlugAddr {
            direction: PlugDirection::Input,
            mode: PlugAddrMode::FunctionBlock(FunctionBlockPlugData {
                fb_type: 0x1f,
                fb_id: 0x07,
                plug_id: 0x29,
            }),
        };
        let raw: [u8;5] = fb.into();
        assert_eq!(fb, raw[..].into());
    }

    #[test]
    fn single_operands() {
        let plug_addr = PlugAddr{
            direction: PlugDirection::Output,
            mode: PlugAddrMode::Unit(UnitPlugData{
                unit_type: UnitPlugType::Pcr,
                plug_id: 0x03,
            }),
        };
        let mut op = ExtendedStreamFormatSingle::new(&plug_addr);
        let mut operands = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut operands).unwrap();
        assert_eq!(&operands, &[0xc0, 0x01, 0x00, 0x00, 0x03, 0xff, 0xff]);

        let operands = [0xc0, 0x01, 0x00, 0x00, 0x03, 0xff, 0x01,
                        0x90, 0x40, 0x04, 0x00, 0x02, 0x02, 0x06, 0x02, 0x00];
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.op.plug_addr, plug_addr);
        assert_eq!(op.op.support_status, SupportStatus::Inactive);

        if let StreamFormat::Am(stream_format) = &op.stream_format {
            if let AmStream::CompoundAm824(s) = stream_format {
                assert_eq!(s.freq, 48000);
                assert_eq!(s.sync_src, false);
                assert_eq!(s.rate_ctl, RateCtl::Supported);
                assert_eq!(s.entries.len(), 2);
                assert_eq!(s.entries[0], CompoundAm824StreamEntry{count: 2, format: CompoundAm824StreamFormat::MultiBitLinearAudioRaw});
                assert_eq!(s.entries[1], CompoundAm824StreamEntry{count: 2, format: CompoundAm824StreamFormat::Iec60958_3});
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }

        let mut operands = Vec::new();
        AvcControl::build_operands(&mut op, &AvcAddr::Unit, &mut operands).unwrap();
        assert_eq!(&operands, &[0xc0, 0x01, 0x00, 0x00, 0x03, 0xff, 0x01,
                                0x90, 0x40, 0x04, 0x00, 0x02, 0x02, 0x06, 0x02, 0x00]);

        let mut op = ExtendedStreamFormatSingle::new(&plug_addr);
        let operands = [0xc0, 0x01, 0x00, 0x00, 0x03, 0xff, 0xff,
                        0x90, 0x40, 0x05, 0x04, 0x02, 0x02, 0x06, 0x02, 0x00];
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.op.plug_addr, plug_addr);
        assert_eq!(op.op.support_status, SupportStatus::NoInfo);
        if let StreamFormat::Am(stream_format) = &op.stream_format {
            if let AmStream::CompoundAm824(s) = stream_format {
                assert_eq!(s.freq, 96000);
                assert_eq!(s.sync_src, true);
                assert_eq!(s.rate_ctl, RateCtl::Supported);
                assert_eq!(s.entries.len(), 2);
                assert_eq!(s.entries[0], CompoundAm824StreamEntry{count: 2, format: CompoundAm824StreamFormat::MultiBitLinearAudioRaw});
                assert_eq!(s.entries[1], CompoundAm824StreamEntry{count: 2, format: CompoundAm824StreamFormat::Iec60958_3});
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }
    }

    #[test]
    fn list_operands() {
        let plug_addr = PlugAddr{
            direction: PlugDirection::Output,
            mode: PlugAddrMode::Unit(UnitPlugData{
                unit_type: UnitPlugType::Pcr,
                plug_id: 0x03,
            }),
        };
        let mut op = ExtendedStreamFormatList::new(&plug_addr, 0x31);
        let mut operands = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut operands).unwrap();
        assert_eq!(&operands, &[0xc1, 0x01, 0x00, 0x00, 0x03, 0xff, 0xff, 0x31]);

        let operands = [0xc1, 0x01, 0x00, 0x00, 0x03, 0xff, 0x01, 0x31,
                        0x90, 0x40, 0x04, 0x00, 0x02, 0x02, 0x06, 0x02, 0x00];
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
                assert_eq!(s.entries[0], CompoundAm824StreamEntry{count: 2, format: CompoundAm824StreamFormat::MultiBitLinearAudioRaw});
                assert_eq!(s.entries[1], CompoundAm824StreamEntry{count: 2, format: CompoundAm824StreamFormat::Iec60958_3});
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }
    }
}
