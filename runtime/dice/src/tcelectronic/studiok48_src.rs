// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*,
    protocols::tcat::global_section::ClockRate,
    protocols::tcelectronic::studio::{SrcEntry, STUDIO_SRC_ENTRIES},
};

/// Assignable sources for mixer-input, channel-strip, and user-assignment controls.
///
/// Same ordering as upstream `SRC_PAIR_ENTRIES` with Stream-B 12/13 added. Excludes mixer buses.
pub const STUDIO_MIXER_SRC_ENTRIES: [SrcEntry; 53] = [
    SrcEntry::Unused,
    SrcEntry::Analog(0),
    SrcEntry::Analog(1),
    SrcEntry::Analog(2),
    SrcEntry::Analog(3),
    SrcEntry::Analog(4),
    SrcEntry::Analog(5),
    SrcEntry::Analog(6),
    SrcEntry::Analog(7),
    SrcEntry::Analog(8),
    SrcEntry::Analog(9),
    SrcEntry::Analog(10),
    SrcEntry::Analog(11),
    SrcEntry::Spdif(0),
    SrcEntry::Spdif(1),
    SrcEntry::Adat(0),
    SrcEntry::Adat(1),
    SrcEntry::Adat(2),
    SrcEntry::Adat(3),
    SrcEntry::Adat(4),
    SrcEntry::Adat(5),
    SrcEntry::Adat(6),
    SrcEntry::Adat(7),
    SrcEntry::StreamA(0),
    SrcEntry::StreamA(1),
    SrcEntry::StreamA(2),
    SrcEntry::StreamA(3),
    SrcEntry::StreamA(4),
    SrcEntry::StreamA(5),
    SrcEntry::StreamA(6),
    SrcEntry::StreamA(7),
    SrcEntry::StreamA(8),
    SrcEntry::StreamA(9),
    SrcEntry::StreamA(10),
    SrcEntry::StreamA(11),
    SrcEntry::StreamA(12),
    SrcEntry::StreamA(13),
    SrcEntry::StreamA(14),
    SrcEntry::StreamA(15),
    SrcEntry::StreamB(0),
    SrcEntry::StreamB(1),
    SrcEntry::StreamB(2),
    SrcEntry::StreamB(3),
    SrcEntry::StreamB(4),
    SrcEntry::StreamB(5),
    SrcEntry::StreamB(6),
    SrcEntry::StreamB(7),
    SrcEntry::StreamB(8),
    SrcEntry::StreamB(9),
    SrcEntry::StreamB(10),
    SrcEntry::StreamB(11),
    SrcEntry::StreamB(12),
    SrcEntry::StreamB(13),
];

/// Assignable sources for physical output routing. Matches [`STUDIO_SRC_ENTRIES`] in protocol.
pub const STUDIO_PHYS_OUT_SRC_ENTRIES: [SrcEntry; 61] = STUDIO_SRC_ENTRIES;

/// Returns whether FW stream A channels 12–13 carry optical (TOS) at the given rate.
pub fn studio_src_stream_a_high_rate(rate: ClockRate) -> bool {
    matches!(
        rate,
        ClockRate::R176400 | ClockRate::R192000 | ClockRate::AnyHigh
    )
}

fn studio_stream_a_to_string(ch: usize, high_rate: bool) -> String {
    match ch {
        0..=11 => format!("Stream-A-{}", ch + 1),
        12 if high_rate => "Stream-A-13-TOS".to_string(),
        13 if high_rate => "Stream-A-14-TOS".to_string(),
        12 | 13 => format!("Stream-A-{}-unused", ch + 1),
        14 => "Stream-A-15-S/PDIF-coax".to_string(),
        15 => "Stream-A-16-S/PDIF-coax".to_string(),
        ch => format!("Stream-A-{}", ch + 1),
    }
}

fn src_entry_labels_for_table(table: &[SrcEntry], rate: ClockRate) -> Vec<String> {
    table
        .iter()
        .map(|entry| studio_src_entry_to_string_for_rate(*entry, rate))
        .collect()
}

/// Human-readable labels for mixer-input and related controls at the given sampling rate.
pub fn studio_mixer_src_entry_labels_for_rate(rate: ClockRate) -> Vec<String> {
    src_entry_labels_for_table(&STUDIO_MIXER_SRC_ENTRIES, rate)
}

/// Human-readable labels for output-source at the given sampling rate.
pub fn studio_phys_out_src_entry_labels_for_rate(rate: ClockRate) -> Vec<String> {
    src_entry_labels_for_table(&STUDIO_PHYS_OUT_SRC_ENTRIES, rate)
}

pub fn studio_src_entry_to_string_for_rate(entry: SrcEntry, rate: ClockRate) -> String {
    let high_rate = studio_src_stream_a_high_rate(rate);
    match entry {
        SrcEntry::StreamA(ch) => studio_stream_a_to_string(ch, high_rate),
        _ => studio_src_entry_to_string(entry),
    }
}

pub fn studio_src_entry_to_string(entry: SrcEntry) -> String {
    match entry {
        SrcEntry::Unused => "Unused".to_string(),
        SrcEntry::Analog(ch) => format!("Analog-{}", ch + 1),
        SrcEntry::Spdif(ch) => format!("S/PDIF-{}", ch + 1),
        SrcEntry::Adat(ch) => format!("ADAT-{}", ch + 1),
        SrcEntry::StreamA(ch) => format!("Stream-A-{}", ch + 1),
        SrcEntry::StreamB(ch) => format!("Stream-B-{}", ch + 1),
        SrcEntry::Mixer(ch) => {
            if ch < 2 {
                format!("Mixer-{}", ch + 1)
            } else if ch < 6 {
                format!("Aux-{}", ch - 1)
            } else {
                format!("Reverb-{}", ch - 5)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn studio_src_entry_stream_a_labels() {
        let low = ClockRate::R48000;
        let high = ClockRate::R192000;

        assert_eq!(
            studio_src_entry_to_string_for_rate(SrcEntry::StreamA(11), low),
            "Stream-A-12"
        );
        assert_eq!(
            studio_src_entry_to_string_for_rate(SrcEntry::StreamA(12), low),
            "Stream-A-13-unused"
        );
        assert_eq!(
            studio_src_entry_to_string_for_rate(SrcEntry::StreamA(13), low),
            "Stream-A-14-unused"
        );
        assert_eq!(
            studio_src_entry_to_string_for_rate(SrcEntry::StreamA(12), high),
            "Stream-A-13-TOS"
        );
        assert_eq!(
            studio_src_entry_to_string_for_rate(SrcEntry::StreamA(13), high),
            "Stream-A-14-TOS"
        );
        assert_eq!(
            studio_src_entry_to_string_for_rate(SrcEntry::StreamA(14), low),
            "Stream-A-15-S/PDIF-coax"
        );
        assert_eq!(
            studio_src_entry_to_string_for_rate(SrcEntry::StreamA(15), high),
            "Stream-A-16-S/PDIF-coax"
        );
        assert_eq!(
            studio_src_entry_to_string_for_rate(SrcEntry::Analog(0), high),
            "Analog-1"
        );

        let mid = ClockRate::R96000;
        assert!(!studio_src_stream_a_high_rate(mid));
        assert_eq!(
            studio_src_entry_to_string_for_rate(SrcEntry::StreamA(12), mid),
            "Stream-A-13-unused"
        );
        assert_eq!(
            studio_mixer_src_entry_labels_for_rate(ClockRate::R48000).len(),
            STUDIO_MIXER_SRC_ENTRIES.len()
        );
        assert_eq!(
            studio_phys_out_src_entry_labels_for_rate(ClockRate::R48000).len(),
            STUDIO_PHYS_OUT_SRC_ENTRIES.len()
        );
    }
}
