// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for M-Audio FireWire series.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by M-Audio normal FireWire series.
//!
//! DM1000 is used for M-Audio FireWire 410. DM1000E is used for M-Audio FireWire Audiophile,
//! and Solo.
//!
//! ## Diagram of internal signal flow for FireWire 410
//!
//! ```text
//! analog-input-1/2 ---+----------------------+--------------------------> stream-output-1/2
//! digital-input-1/2 --|-+--------------------|-+------------------------> stream-output-3/4
//!                     | |                    | |
//!                     | |                    v v
//!                     | |                ++=======++
//!  stream-input-1/2 --|-|-+------------->||       ||
//!  stream-input-3/4 --|-|-|-+----------->|| 14x 2 ||
//!  stream-input-5/6 --|-|-|-|-+--------->||  aux  || ---> aux-output-1/2
//!  stream-input-7/8 --|-|-|-|-|-+------->|| mixer ||        | | | | | |
//!  stream-input-9/10 -|-|-|-|-|-|-+----->||       ||        | | | | | |
//!                     | | | | | | |      ++=======++        +-|-|-|-|-|-> analog-output-1/2
//!                     | | | | | | |                         | | | | | |
//!                     | | | | | | |      ++=======++        | +-|-|-|-|-> analog-output-3/4
//!                     | | +-|-|-|-|----> ||       ||        | | | | | |
//!                     | | | +-|-|-|----> || 10x 2 ||        | | +-|-|-|-> analog-output-5/6
//!                     | | | | +-|-|----> ||  hp   ||        | | | | | |
//!                     | | | | | +-|----> || mixer ||        | | | +-|-|-> analog-output-7/8
//!                     | | | | | | +----> ||       ||        | | | | | |
//!                     | | | | | | |      ++=======++        | | | | +-|-> digital-output-1/2
//!                     | | | | | | |           v             | | | | | |
//!                     | | | | | | |    hp-mixer-output-1/2 -|-|-|-|-|-+-> headphone-1/2
//!                     v v v v v v v                         | | | | |
//!                   ++=============++                       | | | | |
//!                   ||             || -- mixer-output-1/2 --+ | | | |
//!                   ||    14x10    || -- mixer-output-3/4 ----+ | | |
//!                   ||             || -- mixer-output-5/6 ------+ | |
//!                   ||    mixer    || -- mixer-output-7/8 --------+ |
//!                   ||             || -- mixer-output-9/10 ---------+
//!                   ++=============++
//! ```
//!
//! ## Diagram of internal signal flow for FireWire Audiophile
//!
//! ```text
//! analog-input-1/2 ---+----------------------+----------------------> stream-output-1/2
//! digital-input-1/2 --|-+--------------------|-+--------------------> stream-output-3/4
//!                     | |                    | |
//!                     | |                    v v
//!                     | |                ++=======++
//!  stream-input-1/2 --|-|-+------------->|| 10x2  ||
//!  stream-input-3/4 --|-|-|-+----------->||  aux  || --> aux-output-1/2
//!  stream-input-5/6 --|-|-|-|-+--------->|| mixer ||        | | | |
//!                     | | | | |          ++=======++        +-|-|-|-> analog-output-1/2
//!                     | | | | |                             | | | |
//!                     | | | | |                             | +-|-|-> analog-output-3/4
//!                     | | | | |                             | | | |
//!                     | | | | |                             | | +-|-> analog-output-5/6
//!                     | | | | |                             | | | |
//!                     | | | | |                             | | | +-> headphone-1/2
//!                     v v v v v                             | | |     (one source only)
//!                   ++=============++                       | | |       ^   ^   ^
//!                   ||    10x6     || -- mixer-output-1/2 --+-|-|-------+   |   |
//!                   ||    mixer    || -- mixer-output-3/4 ----+-|-----------+   |
//!                   ||             || -- mixer-output-5/6 ------+---------------+
//!                   ++=============++
//! ```
//!
//! ## Diagram of internal signal flow for FireWire Solo
//!
//! ```text
//! analog-input-1/2 --------+------------------------------> stream-output-1/2
//! digital-input-1/2 -------|-+----------------------------> stream-output-3/4
//!                          | |
//!                          v v
//!                      ++=======++
//!  stream-input-1/2 -->||  8x4  || --> mixer-output-1/2 --> analog-output-1/2
//!  stream-input-3/4 -->|| mixer || --> mixer-output-3/4 --> digital-output-1/2
//!                      ++=======++
//! ```
//!
//! ## Diagram of internal signal flow for Ozonic
//!
//! ```text
//! analog-input-1/2 --------+------------------------------> stream-output-1/2
//! analog-input-3/4 --------|-+----------------------------> stream-output-3/4
//!                          | |
//!                          v v
//!                      ++=======++
//!  stream-input-1/2 -->||  8x4  || --> mixer-output-1/2 --> analog-output-1/2
//!  stream-input-3/4 -->|| mixer || --> mixer-output-3/4 --> analog-output-3/4
//!                      ++=======++
//! ```
//!
//! The protocol implementation for M-Audio FireWire 410 was written with firmware version below:
//!
//! ```sh
//! $ cargo run --bin bco-bootloader-info -- /dev/fw1
//! protocol:
//!   version: 1
//! bootloader:
//!   timestamp: 2003-04-04T01:46:25+0000
//!   version: 0.0.0
//! hardware:
//!   GUID: 0x000af510000d6c01
//!   model ID: 0x000002
//!   revision: 0.0.1
//! software:
//!   timestamp: 2007-05-04T10:26:56+0000
//!   ID: 0x00010046
//!   revision: 0.255.65535
//! image:
//!   base address: 0x20080000
//!   maximum size: 0x180000
//! ```
//!
//! The protocol implementation for M-Audio FireWire Audiophile was written with firmware version
//! below:
//!
//! ```sh
//! $ cargo run --bin bco-bootloader-info -- /dev/fw1
//! protocol:
//!   version: 1
//! bootloader:
//!   timestamp: 2003-10-06T10:00:41+0000
//!   version: 0.0.0
//! hardware:
//!   GUID: 0x002b7e2e000d6c03
//!   model ID: 0x00000d
//!   revision: 0.0.1
//! software:
//!   timestamp: 2007-05-04T10:22:12+0000
//!   ID: 0x00010060
//!   revision: 0.255.65535
//! image:
//!   base address: 0x20080000
//!   maximum size: 0x180000
//! ```
//!
//! The protocol implementation for M-Audio FireWire Solo was written with firmware version below:
//!
//! ```sh
//! $ cargo run --bin bco-bootloader-info -- /dev/fw1
//! protocol:
//!   version: 1
//! bootloader:
//!   timestamp: 2004-09-15T01:22:54+0000
//!   version: 0.0.0
//! hardware:
//!   GUID: 0x00c256a4000d6c0b
//!   model ID: 0x000090
//!   revision: 0.0.0
//! software:
//!   timestamp: 2007-08-08T01:56:28+0000
//!   ID: 0x00010062
//!   revision: 0.255.65535
//! image:
//!   base address: 0x20080000
//!   maximum size: 0x180000
//! ```
//!
//! The protocol implementation for M-Audio Ozonic was written with firmware version below:
//!
//! ```sh
//! $ cargo run --bin bco-bootloader-info -- /dev/fw1
//! protocol:
//!   version: 1
//! bootloader:
//!   timestamp: 2004-09-15T05:56:44+0000
//!   version: 0.0.0
//! hardware:
//!   GUID: 0x0078c1d7000d6c0a
//!   model ID: 0x000001
//!   revision: 0.0.1
//! software:
//!   timestamp: 2005-09-05T09:10:22+0000
//!   ID: 0x0000000a
//!   revision: 0.0.20
//! image:
//!   base address: 0x20080000
//!   maximum size: 0x180000
//! ```

use super::*;

/// The protocol implementation for media and sampling clock of FireWire 410.
#[derive(Default, Debug)]
pub struct Fw410ClkProtocol;

impl MediaClockFrequencyOperation for Fw410ClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000, 176400, 192000];
}

impl SamplingClockSourceOperation for Fw410ClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x01,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal
        SignalAddr::Subunit(SignalSubunitAddr {
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x01,
        }),
        // S/PDIF
        SignalAddr::Unit(SignalUnitAddr::Ext(0x02)),
    ];
}

/// The protocol implementation for meter in FireWire 410.
#[derive(Default, Debug)]
pub struct Fw410MeterProtocol;

impl MaudioNormalMeterProtocol for Fw410MeterProtocol {
    const PHYS_INPUT_COUNT: usize = 4;
    const STREAM_INPUT_COUNT: usize = 0;
    const PHYS_OUTPUT_COUNT: usize = 10;
    const ROTARY_COUNT: usize = 1;
    const HAS_SWITCH: bool = false;
    const HAS_SYNC_STATUS: bool = true;
}

/// The protocol implementation for physical input of FireWire 410.
#[derive(Default, Debug)]
pub struct Fw410PhysInputProtocol;

impl AvcAudioFeatureSpecification for Fw410PhysInputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x03, AudioCh::Each(0)), // analog-input-1
        (0x03, AudioCh::Each(1)), // analog-input-2
        (0x04, AudioCh::Each(0)), // digital-input-1
        (0x04, AudioCh::Each(1)), // digital-input-2
    ];
}

impl AvcLevelOperation for Fw410PhysInputProtocol {}

impl AvcLrBalanceOperation for Fw410PhysInputProtocol {}

/// The protocol implementation for physical output of FireWire 410.
#[derive(Default, Debug)]
pub struct Fw410PhysOutputProtocol;

impl AvcAudioFeatureSpecification for Fw410PhysOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x0a, AudioCh::Each(0)), // analog-output-1
        (0x0a, AudioCh::Each(1)), // analog-output-2
        (0x0b, AudioCh::Each(0)), // analog-output-3
        (0x0b, AudioCh::Each(1)), // analog-output-4
        (0x0c, AudioCh::Each(0)), // analog-output-5
        (0x0c, AudioCh::Each(1)), // analog-output-6
        (0x0d, AudioCh::Each(0)), // analog-output-7
        (0x0d, AudioCh::Each(1)), // analog-output-8
        (0x0e, AudioCh::Each(0)), // digital-output-1
        (0x0e, AudioCh::Each(1)), // digital-output-2
    ];
}

impl AvcLevelOperation for Fw410PhysOutputProtocol {}

impl AvcSelectorOperation for Fw410PhysOutputProtocol {
    // NOTE: "analog-output-1/2", "analog-output-3/4", "analog-output-5/6", "analog-output-7/8",
    //       "analog-output-9/10"
    const FUNC_BLOCK_ID_LIST: &'static [u8] = &[0x02, 0x03, 0x04, 0x05, 0x06];
    // NOTE: "mixer-output", "aux-output-1/2"
    const INPUT_PLUG_ID_LIST: &'static [u8] = &[0x00, 0x01];
}

/// The protocol implementation for source of aux mixer in FireWire 410.
#[derive(Default, Debug)]
pub struct Fw410AuxSourceProtocol;

impl AvcAudioFeatureSpecification for Fw410AuxSourceProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x07, AudioCh::Each(0)), // analog-input-1
        (0x07, AudioCh::Each(1)), // analog-input-2
        (0x08, AudioCh::Each(0)), // digital-input-1
        (0x08, AudioCh::Each(1)), // digital-input-2
        (0x06, AudioCh::Each(0)), // stream-input-1
        (0x06, AudioCh::Each(1)), // stream-input-2
        (0x05, AudioCh::Each(0)), // stream-input-3
        (0x05, AudioCh::Each(1)), // stream-input-4
        (0x05, AudioCh::Each(2)), // stream-input-5
        (0x05, AudioCh::Each(3)), // stream-input-6
        (0x05, AudioCh::Each(4)), // stream-input-7
        (0x05, AudioCh::Each(5)), // stream-input-8
        (0x05, AudioCh::Each(6)), // stream-input-9
        (0x05, AudioCh::Each(7)), // stream-input-10
    ];
}

impl AvcLevelOperation for Fw410AuxSourceProtocol {}

/// The protocol implementation for output of aux mixer in FireWire 410.
#[derive(Default, Debug)]
pub struct Fw410AuxOutputProtocol;

impl AvcAudioFeatureSpecification for Fw410AuxOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x09, AudioCh::Each(0)), // aux-output-1
        (0x09, AudioCh::Each(1)), // aux-output-2
    ];
}

impl AvcLevelOperation for Fw410AuxOutputProtocol {}

/// The protocol implementation for output of headphone in FireWire 410.
#[derive(Default, Debug)]
pub struct Fw410HeadphoneProtocol;

impl AvcAudioFeatureSpecification for Fw410HeadphoneProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x0f, AudioCh::Each(0)), // headphone-1
        (0x0f, AudioCh::Each(1)), // headphone-2
    ];
}

impl AvcLevelOperation for Fw410HeadphoneProtocol {}

impl AvcSelectorOperation for Fw410HeadphoneProtocol {
    const FUNC_BLOCK_ID_LIST: &'static [u8] = &[0x07];
    // NOTE: "mixer", "aux-1/2".
    const INPUT_PLUG_ID_LIST: &'static [u8] = &[0x00, 0x01];
}

/// The protocol implementation for source of S/PDIF output in FireWire 410.
#[derive(Default, Debug)]
pub struct Fw410SpdifOutputProtocol;

impl AvcSelectorOperation for Fw410SpdifOutputProtocol {
    const FUNC_BLOCK_ID_LIST: &'static [u8] = &[0x01];
    // NOTE: "Coaxial", "Optical".
    const INPUT_PLUG_ID_LIST: &'static [u8] = &[0x00, 0x01];
}

/// The protocol implementation for mixer in FireWire 410.
#[derive(Default, Debug)]
pub struct Fw410MixerProtocol;

impl MaudioNormalMixerOperation for Fw410MixerProtocol {
    const DST_FUNC_BLOCK_ID_LIST: &'static [(u8, AudioCh)] = &[
        (0x01, AudioCh::Each(0)), // mixer-1/2
        (0x01, AudioCh::Each(2)), // mixer-3/4
        (0x01, AudioCh::Each(4)), // mixer-5/6
        (0x01, AudioCh::Each(6)), // mixer-7/8
        (0x01, AudioCh::Each(8)), // mixer-1/2
    ];
    const SRC_FUNC_BLOCK_ID_LIST: &'static [(u8, AudioCh)] = &[
        (0x02, AudioCh::Each(0)), // analog-input-1/2
        (0x03, AudioCh::Each(0)), // digital-input-1/2
        (0x01, AudioCh::Each(0)), // stream-input-1/2
        (0x00, AudioCh::Each(0)), // stream-input-3/4
        (0x00, AudioCh::Each(2)), // stream-input-5/6
        (0x00, AudioCh::Each(4)), // stream-input-7/8
        (0x00, AudioCh::Each(6)), // stream-input-9/10
    ];
}

impl MaudioNormalMixerOperation for Fw410HeadphoneProtocol {
    const DST_FUNC_BLOCK_ID_LIST: &'static [(u8, AudioCh)] = &[(0x07, AudioCh::Each(0))];
    const SRC_FUNC_BLOCK_ID_LIST: &'static [(u8, AudioCh)] = &[
        (0x00, AudioCh::Each(0)), // mixer-output-1/2
        (0x00, AudioCh::Each(2)), // mixer-output-3/4
        (0x00, AudioCh::Each(4)), // mixer-output-5/6
        (0x00, AudioCh::Each(6)), // mixer-output-7/8
        (0x00, AudioCh::Each(8)), // mixer-output-9/10
    ];
}

/// The protocol implementation for media and sampling clock of FireWire Solo.
#[derive(Default, Debug)]
pub struct SoloClkProtocol;

impl MediaClockFrequencyOperation for SoloClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

impl SamplingClockSourceOperation for SoloClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x01,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal
        SignalAddr::Subunit(SignalSubunitAddr {
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x01,
        }),
        // S/PDIF
        SignalAddr::Unit(SignalUnitAddr::Ext(0x01)),
    ];
}

/// The protocol implementation for meter in FireWire Solo.
#[derive(Default, Debug)]
pub struct SoloMeterProtocol;

impl MaudioNormalMeterProtocol for SoloMeterProtocol {
    const PHYS_INPUT_COUNT: usize = 4;
    const STREAM_INPUT_COUNT: usize = 4;
    const PHYS_OUTPUT_COUNT: usize = 4;
    const ROTARY_COUNT: usize = 0;
    const HAS_SWITCH: bool = false;
    const HAS_SYNC_STATUS: bool = true;
}

/// The protocol implementation for physical input of FireWire Solo.
#[derive(Default, Debug)]
pub struct SoloPhysInputProtocol;

impl AvcAudioFeatureSpecification for SoloPhysInputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x03, AudioCh::Each(0)), // analog-input-1
        (0x03, AudioCh::Each(1)), // analog-input-2
        (0x04, AudioCh::Each(0)), // digital-input-1
        (0x04, AudioCh::Each(1)), // digital-input-2
    ];
}

impl AvcLevelOperation for SoloPhysInputProtocol {}

impl AvcLrBalanceOperation for SoloPhysInputProtocol {}

/// The protocol implementation for stream input of FireWire Solo.
#[derive(Default, Debug)]
pub struct SoloStreamInputProtocol;

impl AvcAudioFeatureSpecification for SoloStreamInputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x01, AudioCh::Each(0)), // stream-input-1
        (0x01, AudioCh::Each(1)), // stream-input-2
        (0x02, AudioCh::Each(0)), // stream-input-3
        (0x02, AudioCh::Each(1)), // stream-input-4
    ];
}

impl AvcLevelOperation for SoloStreamInputProtocol {}

// NOTE: outputs are not configurable, connected to hardware dial directly.

/// The protocol implementation for source of S/PDIF output in FireWire Solo.
#[derive(Default, Debug)]
pub struct SoloSpdifOutputProtocol;

impl AvcSelectorOperation for SoloSpdifOutputProtocol {
    const FUNC_BLOCK_ID_LIST: &'static [u8] = &[0x01];
    // NOTE: "stream-3/4", "mixer-3/4".
    const INPUT_PLUG_ID_LIST: &'static [u8] = &[0x00, 0x01];
}

/// The protocol implementation for mixer in FireWire Solo.
#[derive(Default, Debug)]
pub struct SoloMixerProtocol;

impl MaudioNormalMixerOperation for SoloMixerProtocol {
    const DST_FUNC_BLOCK_ID_LIST: &'static [(u8, AudioCh)] = &[
        // mixer-1/2 directly connected to analog-output-1/2 and headphone-1/2
        (0x01, AudioCh::Each(0)),
        (0x01, AudioCh::Each(2)), // mixer-3/4 directly connected to digital-output-1/2
    ];
    const SRC_FUNC_BLOCK_ID_LIST: &'static [(u8, AudioCh)] = &[
        (0x00, AudioCh::Each(0)), // analog-input-1/2
        (0x01, AudioCh::Each(0)), // digital-input-1/2
        (0x02, AudioCh::Each(0)), // stream-input-1/2
        (0x03, AudioCh::Each(0)), // stream-input-3/4
    ];
}

/// The protocol implementation for media and sampling clock of FireWire Audiophile.
#[derive(Default, Debug)]
pub struct AudiophileClkProtocol;

impl MediaClockFrequencyOperation for AudiophileClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

impl SamplingClockSourceOperation for AudiophileClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x01,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal
        SignalAddr::Subunit(SignalSubunitAddr {
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x01,
        }),
        // S/PDIF
        SignalAddr::Unit(SignalUnitAddr::Ext(0x02)),
    ];
}

/// The protocol implementation for meter in FireWire Audiophile.
#[derive(Default, Debug)]
pub struct AudiophileMeterProtocol;

impl MaudioNormalMeterProtocol for AudiophileMeterProtocol {
    const PHYS_INPUT_COUNT: usize = 4;
    const STREAM_INPUT_COUNT: usize = 0;
    const PHYS_OUTPUT_COUNT: usize = 6;
    const ROTARY_COUNT: usize = 2;
    const HAS_SWITCH: bool = true;
    const HAS_SYNC_STATUS: bool = true;
}

/// The protocol implementation for physical input in FireWire Audiophile.
#[derive(Default, Debug)]
pub struct AudiophilePhysInputProtocol;

impl AvcAudioFeatureSpecification for AudiophilePhysInputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x04, AudioCh::Each(0)), // analog-input-1
        (0x04, AudioCh::Each(1)), // analog-input-2
        (0x05, AudioCh::Each(0)), // digital-input-1
        (0x05, AudioCh::Each(1)), // digital-input-2
    ];
}

impl AvcLevelOperation for AudiophilePhysInputProtocol {}

impl AvcLrBalanceOperation for AudiophilePhysInputProtocol {}

/// The protocol implementation for physical output in FireWire Audiophile.
#[derive(Default, Debug)]
pub struct AudiophilePhysOutputProtocol;

impl AvcAudioFeatureSpecification for AudiophilePhysOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x0c, AudioCh::Each(0)), // analog-output-1
        (0x0c, AudioCh::Each(1)), // analog-output-2
        (0x0d, AudioCh::Each(0)), // analog-output-3
        (0x0d, AudioCh::Each(1)), // analog-output-4
        (0x0e, AudioCh::Each(0)), // digital-output-1
        (0x0e, AudioCh::Each(1)), // digital-output-2
    ];
}

impl AvcLevelOperation for AudiophilePhysOutputProtocol {}

impl AvcSelectorOperation for AudiophilePhysOutputProtocol {
    // NOTE: "analog-output-1/2", "analog-output-3/4", "analog-output-5/6"
    const FUNC_BLOCK_ID_LIST: &'static [u8] = &[0x01, 0x02, 0x03];
    // NOTE: "mixer-output", "aux-output-1/2"
    const INPUT_PLUG_ID_LIST: &'static [u8] = &[0x00, 0x01];
}

/// The protocol implementation for source of aux mixer in FireWire Audiophile.
#[derive(Default, Debug)]
pub struct AudiophileAuxSourceProtocol;

impl AvcAudioFeatureSpecification for AudiophileAuxSourceProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x09, AudioCh::Each(0)), // analog-input-1
        (0x09, AudioCh::Each(1)), // analog-input-2
        (0x0a, AudioCh::Each(0)), // digital-input-1
        (0x0a, AudioCh::Each(1)), // digital-input-2
        (0x06, AudioCh::Each(0)), // stream-input-1
        (0x06, AudioCh::Each(1)), // stream-input-2
        (0x07, AudioCh::Each(0)), // stream-input-3
        (0x07, AudioCh::Each(1)), // stream-input-4
        (0x08, AudioCh::Each(0)), // stream-input-5
        (0x08, AudioCh::Each(1)), // stream-input-6
    ];
}

impl AvcLevelOperation for AudiophileAuxSourceProtocol {}

/// The protocol implementation for output of aux mixer in FireWire Audiophile.
#[derive(Default, Debug)]
pub struct AudiophileAuxOutputProtocol;

impl AvcAudioFeatureSpecification for AudiophileAuxOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x0b, AudioCh::Each(0)), // aux-output-1
        (0x0b, AudioCh::Each(1)), // aux-output-2
    ];
}

impl AvcLevelOperation for AudiophileAuxOutputProtocol {}

/// The protocol implementation for output of headphone in FireWire Audiophile.
#[derive(Default, Debug)]
pub struct AudiophileHeadphoneProtocol;

impl AvcAudioFeatureSpecification for AudiophileHeadphoneProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x0f, AudioCh::Each(0)), // headphone-1
        (0x0f, AudioCh::Each(1)), // headphone-2
    ];
}

impl AvcLevelOperation for AudiophileHeadphoneProtocol {}

impl AvcSelectorOperation for AudiophileHeadphoneProtocol {
    const FUNC_BLOCK_ID_LIST: &'static [u8] = &[0x04];
    // NOTE: "mixer-1/2", "mixer-3/4", "mixer-5/6", "aux-1/2".
    const INPUT_PLUG_ID_LIST: &'static [u8] = &[0x00, 0x01, 0x02, 0x03];
}

/// The protocol implementation for mixer in FireWire Solo.
#[derive(Default, Debug)]
pub struct AudiophileMixerProtocol;

impl MaudioNormalMixerOperation for AudiophileMixerProtocol {
    const DST_FUNC_BLOCK_ID_LIST: &'static [(u8, AudioCh)] = &[
        (0x01, AudioCh::Each(0)), // mixer-1/2
        (0x02, AudioCh::Each(0)), // mixer-3/4
        (0x03, AudioCh::Each(0)), // mixer-5/6
    ];
    const SRC_FUNC_BLOCK_ID_LIST: &'static [(u8, AudioCh)] = &[
        (0x03, AudioCh::Each(0)), // analog-input-1/2
        (0x04, AudioCh::Each(0)), // digital-input-1/2
        (0x00, AudioCh::Each(0)), // stream-input-1/2
        (0x01, AudioCh::Each(0)), // stream-input-3/4
        (0x02, AudioCh::Each(0)), // stream-input-5/6
    ];
}

/// The protocol implementation for media and sampling clock of Ozonic.
#[derive(Default, Debug)]
pub struct OzonicClkProtocol;

impl MediaClockFrequencyOperation for OzonicClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

impl SamplingClockSourceOperation for OzonicClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x05,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal
        SignalAddr::Subunit(SignalSubunitAddr {
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x05,
        }),
    ];
}

/// The protocol implementation for meter in Ozonic.
#[derive(Default, Debug)]
pub struct OzonicMeterProtocol;

impl MaudioNormalMeterProtocol for OzonicMeterProtocol {
    const PHYS_INPUT_COUNT: usize = 4;
    const STREAM_INPUT_COUNT: usize = 4;
    const PHYS_OUTPUT_COUNT: usize = 4;
    const ROTARY_COUNT: usize = 0;
    const HAS_SWITCH: bool = false;
    const HAS_SYNC_STATUS: bool = false;
}

/// The protocol implementation for physical input of FireWire Audiophile.
#[derive(Default, Debug)]
pub struct OzonicPhysInputProtocol;

impl AvcAudioFeatureSpecification for OzonicPhysInputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x03, AudioCh::Each(0)), // analog-input-1
        (0x03, AudioCh::Each(1)), // analog-input-2
        (0x04, AudioCh::Each(0)), // analog-input-3
        (0x04, AudioCh::Each(1)), // analog-input-4
    ];
}

impl AvcLevelOperation for OzonicPhysInputProtocol {}

impl AvcLrBalanceOperation for OzonicPhysInputProtocol {}

/// The protocol implementation for stream input of FireWire Audiophile.
#[derive(Default, Debug)]
pub struct OzonicStreamInputProtocol;

impl AvcAudioFeatureSpecification for OzonicStreamInputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x01, AudioCh::Each(0)), // stream-input-1
        (0x01, AudioCh::Each(1)), // stream-input-2
        (0x02, AudioCh::Each(0)), // stream-input-3
        (0x02, AudioCh::Each(1)), // stream-input-4
    ];
}

impl AvcLevelOperation for OzonicStreamInputProtocol {}

// NOTE: outputs are not configurable, connected to hardware dial directly.

/// The state of switch with LED specific to FireWire Audiophile.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AudiophileSwitchState {
    Off,
    A,
    B,
}

impl Default for AudiophileSwitchState {
    fn default() -> Self {
        Self::Off
    }
}

impl AudiophileSwitchState {
    const VALUE_OFF: u8 = 0x00;
    const VALUE_A: u8 = 0x01;
    const VALUE_B: u8 = 0x02;

    fn to_val(&self) -> u8 {
        match self {
            Self::Off => Self::VALUE_OFF,
            Self::A => Self::VALUE_A,
            Self::B => Self::VALUE_B,
        }
    }

    #[allow(dead_code)]
    fn from_val(val: u8) -> Self {
        match val {
            Self::VALUE_A => Self::A,
            Self::VALUE_B => Self::B,
            _ => Self::Off,
        }
    }
}

/// The structure to express AV/C vendor-dependent command for LED switch specific to FireWire
/// Audiophile.
pub struct AudiophileLedSwitch {
    state: AudiophileSwitchState,
    op: VendorDependent,
}

impl AudiophileLedSwitch {
    pub fn new(switch_state: AudiophileSwitchState) -> Self {
        let mut instance = Self::default();
        instance.state = switch_state;
        instance
    }
}

impl Default for AudiophileLedSwitch {
    fn default() -> Self {
        Self {
            state: Default::default(),
            op: VendorDependent {
                company_id: MAUDIO_OUI,
                data: vec![0x02, 0x00, 0x01, 0xff, 0xff, 0xff],
            },
        }
    }
}

impl AvcOp for AudiophileLedSwitch {
    const OPCODE: u8 = VendorDependent::OPCODE;
}

impl AvcControl for AudiophileLedSwitch {
    fn build_operands(&mut self, addr: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
        self.op.data[3] = self.state.to_val();
        AvcControl::build_operands(&mut self.op, addr)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        AvcControl::parse_operands(&mut self.op, addr, operands)
    }
}

/// The structure to express metering information. The `Default` trait should be implemented to
/// call `MaudioNormalMeterProtocol::create_meter()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaudioNormalMeter {
    pub phys_inputs: Vec<i32>,
    pub stream_inputs: Option<Vec<i32>>,
    pub phys_outputs: Vec<i32>,
    pub headphone: Option<[i32; 2]>,
    pub aux_output: Option<[i32; 2]>,
    pub rotaries: Option<[i32; 2]>,
    pub switch: Option<AudiophileSwitchState>,
    pub sync_status: Option<bool>,

    cache: Vec<u8>,
}

/// The trait for meter protocol specific to M-Audio FireWire series.
pub trait MaudioNormalMeterProtocol {
    const PHYS_INPUT_COUNT: usize;
    const STREAM_INPUT_COUNT: usize;
    const PHYS_OUTPUT_COUNT: usize;
    const ROTARY_COUNT: usize;
    const HAS_SWITCH: bool;
    const HAS_SYNC_STATUS: bool;

    const LEVEL_MIN: i32 = 0;
    const LEVEL_MAX: i32 = i32::MAX;
    const LEVEL_STEP: i32 = 0x100;

    const ROTARY_MIN: i32 = 0;
    const ROTARY_MAX: i32 = i16::MAX as i32;
    const ROTARY_STEP: i32 = 0x200;

    fn create_meter() -> MaudioNormalMeter {
        let mut meter = MaudioNormalMeter {
            phys_inputs: vec![0; Self::PHYS_INPUT_COUNT],
            stream_inputs: Default::default(),
            phys_outputs: vec![0; Self::PHYS_OUTPUT_COUNT],
            headphone: Default::default(),
            aux_output: Default::default(),
            rotaries: Default::default(),
            switch: Default::default(),
            sync_status: Default::default(),
            cache: vec![0; Self::calculate_meter_frame_size()],
        };

        if Self::STREAM_INPUT_COUNT > 0 {
            meter.stream_inputs = Some(vec![0; Self::STREAM_INPUT_COUNT]);
        } else {
            meter.headphone = Some(Default::default());
            meter.aux_output = Some(Default::default());
        }

        if Self::ROTARY_COUNT > 0 {
            meter.rotaries = Some(Default::default());
        }

        if Self::HAS_SWITCH {
            meter.switch = Some(Default::default());
        }

        if Self::HAS_SYNC_STATUS {
            meter.sync_status = Some(Default::default());
        }

        meter
    }

    fn calculate_meter_frame_size() -> usize {
        let mut size = Self::PHYS_INPUT_COUNT + Self::PHYS_OUTPUT_COUNT;

        if Self::STREAM_INPUT_COUNT > 0 {
            size += Self::STREAM_INPUT_COUNT;
        } else {
            // Plus headphone-1 and -2, aux-1 and -2.
            size += 4;
        }

        if Self::ROTARY_COUNT > 0 || Self::HAS_SWITCH || Self::HAS_SYNC_STATUS {
            size += 1;
        }

        size * 4
    }

    fn read_meter(
        req: &FwReq,
        node: &FwNode,
        meter: &mut MaudioNormalMeter,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let frame = &mut meter.cache;

        // For rotaries, switch, and sync_status if available.
        let mut bitmap = [0; 4];
        let pos = frame.len() - 4;
        bitmap.copy_from_slice(&frame[pos..]);

        req.transaction(
            node,
            FwTcode::ReadBlockRequest,
            DM_APPL_METER_OFFSET,
            frame.len(),
            frame,
            timeout_ms,
        )?;

        let mut quadlet = [0; 4];

        meter.phys_inputs.iter_mut().enumerate().for_each(|(i, m)| {
            let pos = i * 4;
            quadlet.copy_from_slice(&frame[pos..(pos + 4)]);
            *m = i32::from_be_bytes(quadlet);
        });

        if let Some(stream_inputs) = &mut meter.stream_inputs {
            stream_inputs.iter_mut().enumerate().for_each(|(i, m)| {
                let pos = (Self::PHYS_INPUT_COUNT + i) * 4;
                quadlet.copy_from_slice(&frame[pos..(pos + 4)]);
                *m = i32::from_be_bytes(quadlet);
            });
        }

        meter
            .phys_outputs
            .iter_mut()
            .enumerate()
            .for_each(|(i, m)| {
                let pos = (Self::PHYS_INPUT_COUNT + Self::STREAM_INPUT_COUNT + i) * 4;
                quadlet.copy_from_slice(&frame[pos..(pos + 4)]);
                *m = i32::from_be_bytes(quadlet);
            });

        if let Some(headphone) = &mut meter.headphone {
            headphone.iter_mut().enumerate().for_each(|(i, m)| {
                let pos = (Self::PHYS_INPUT_COUNT + Self::PHYS_OUTPUT_COUNT + i) * 4;
                quadlet.copy_from_slice(&frame[pos..(pos + 4)]);
                *m = i32::from_be_bytes(quadlet);
            });
        }

        if let Some(aux_output) = &mut meter.aux_output {
            aux_output.iter_mut().enumerate().for_each(|(i, m)| {
                let pos = (Self::PHYS_INPUT_COUNT + Self::PHYS_OUTPUT_COUNT + 2 + i) * 4;
                quadlet.copy_from_slice(&frame[pos..(pos + 4)]);
                *m = i32::from_be_bytes(quadlet);
            });
        }

        if let Some(rotaries) = &mut meter.rotaries {
            rotaries.iter_mut().enumerate().for_each(|(i, r)| {
                let shift = i * 4;
                if (bitmap[1] ^ frame[frame.len() - 3]) & (0x0f << shift) > 0 {
                    let flag = (frame[frame.len() - 3] >> shift) & 0x0f;
                    if flag == 0x01 {
                        if *r <= Self::ROTARY_MAX - Self::ROTARY_STEP {
                            *r += Self::ROTARY_STEP;
                        }
                    } else if flag == 0x02 {
                        if *r >= Self::ROTARY_MIN + Self::ROTARY_STEP {
                            *r -= Self::ROTARY_STEP;
                        }
                    }
                }
            });
        }

        if let Some(switch) = &mut meter.switch {
            if bitmap[0] ^ frame[frame.len() - 4] & 0xf0 > 0 {
                if bitmap[0] & 0xf0 > 0 {
                    *switch = match *switch {
                        AudiophileSwitchState::Off => AudiophileSwitchState::A,
                        AudiophileSwitchState::A => AudiophileSwitchState::B,
                        AudiophileSwitchState::B => AudiophileSwitchState::Off,
                    };
                }
            }
        }

        if let Some(sync_status) = &mut meter.sync_status {
            *sync_status = frame[frame.len() - 1] > 0;
        }

        Ok(())
    }
}

/// The protocol implementation for mixer in FireWire Solo.
#[derive(Default, Debug)]
pub struct OzonicMixerProtocol;

impl MaudioNormalMixerOperation for OzonicMixerProtocol {
    const DST_FUNC_BLOCK_ID_LIST: &'static [(u8, AudioCh)] = &[
        (0x01, AudioCh::Each(0)), // mixer-1/2 directly connected to analog-output-1/2
        (0x02, AudioCh::Each(0)), // mixer-3/4 directly connected to analog-output-3/4
    ];
    const SRC_FUNC_BLOCK_ID_LIST: &'static [(u8, AudioCh)] = &[
        (0x02, AudioCh::Each(0)), // analog-input-1/2
        (0x03, AudioCh::Each(0)), // analog-input-3/4
        (0x00, AudioCh::Each(0)), // stream-input-1/2
        (0x01, AudioCh::Each(0)), // stream-input-3/4
    ];
}

/// The parameter of mixer. The `Default` trait should be implemented to call
/// `MaudioNormalMixerOperation::create_mixer_parameters()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaudioNormalMixerParameters(pub Vec<Vec<bool>>);

/// The trait for mixer operation.
pub trait MaudioNormalMixerOperation {
    const DST_FUNC_BLOCK_ID_LIST: &'static [(u8, AudioCh)];
    const SRC_FUNC_BLOCK_ID_LIST: &'static [(u8, AudioCh)];

    const SRC_OFF: i16 = 0x8000u16 as i16;
    const SRC_ON: i16 = 0;

    fn create_mixer_parameters() -> MaudioNormalMixerParameters {
        MaudioNormalMixerParameters(vec![
            vec![
                Default::default();
                Self::SRC_FUNC_BLOCK_ID_LIST.len()
            ];
            Self::DST_FUNC_BLOCK_ID_LIST.len()
        ])
    }

    /// The ASIC get heavy load from AV/C status request for mixer parameters, thus this method
    /// often brings timeout.
    fn cache(
        avc: &BebobAvc,
        params: &mut MaudioNormalMixerParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(params.0.len(), Self::DST_FUNC_BLOCK_ID_LIST.len());
        assert_eq!(params.0[0].len(), Self::SRC_FUNC_BLOCK_ID_LIST.len());

        params
            .0
            .iter_mut()
            .zip(Self::DST_FUNC_BLOCK_ID_LIST)
            .try_for_each(|(srcs, &(dst_func_block_id, dst_audio_ch))| {
                srcs.iter_mut()
                    .zip(Self::SRC_FUNC_BLOCK_ID_LIST)
                    .try_for_each(|(src, &(src_func_block_id, src_audio_ch))| {
                        let mut op = AudioProcessing::new(
                            dst_func_block_id,
                            CtlAttr::Current,
                            src_func_block_id,
                            src_audio_ch,
                            dst_audio_ch,
                            ProcessingCtl::Mixer(vec![-1]),
                        );
                        avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
                            .map(|_| {
                                if let ProcessingCtl::Mixer(data) = op.ctl {
                                    *src = data[0] == Self::SRC_ON
                                }
                            })
                    })
            })
    }

    fn update(
        avc: &BebobAvc,
        params: &MaudioNormalMixerParameters,
        old: &mut MaudioNormalMixerParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        old.0
            .iter_mut()
            .zip(params.0.iter())
            .zip(Self::DST_FUNC_BLOCK_ID_LIST)
            .try_for_each(
                |((old_srcs, new_srcs), &(dst_func_block_id, dst_audio_ch))| {
                    old_srcs
                        .iter_mut()
                        .zip(new_srcs.iter())
                        .zip(Self::SRC_FUNC_BLOCK_ID_LIST)
                        .filter(|((o, n), _)| !n.eq(o))
                        .try_for_each(|((old, &new), &(src_func_block_id, src_audio_ch))| {
                            let val = if new { Self::SRC_ON } else { Self::SRC_OFF };
                            let mut op = AudioProcessing::new(
                                dst_func_block_id,
                                CtlAttr::Current,
                                src_func_block_id,
                                src_audio_ch,
                                dst_audio_ch,
                                ProcessingCtl::Mixer(vec![val]),
                            );
                            avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
                                .map(|_| *old = new)
                        })
                },
            )
    }
}
