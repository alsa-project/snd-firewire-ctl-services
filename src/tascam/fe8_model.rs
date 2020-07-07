// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use super::async_unit::ConsoleData;

pub struct Fe8Model{}

impl<'a> ConsoleData<'a> for Fe8Model {
    const FW_LED: &'a [u16] = &[0x16, 0x8e];

    const SIMPLE_LEDS: &'a [&'a [u16]] = &[
        &[0x05],            // rec-0
        &[0x18, 0x25],      // rec-1
        &[0x38, 0x45],      // rec-2
        &[0x58, 0x65],      // rec-3
        &[0x76, 0x82],      // rec-4
        &[0x98, 0xa5],      // rec-5
        &[0xb8, 0xc5],      // rec-6
        &[0xd8, 0xe5],      // rec-7
    ];

    const TOGGLED_BUTTONS: &'a [((u32, u32), &'a [u16])] = &[
        ((13, 0x00000001), &[0x00]),        // select-0
        ((13, 0x00000002), &[0x13, 0x20]),  // select-1
        ((13, 0x00000004), &[0x33, 0x40]),  // select-2
        ((13, 0x00000008), &[0x53, 0x60]),  // select-3
        ((13, 0x00000010), &[0x73, 0x80]),  // select-4
        ((13, 0x00000020), &[0x93, 0xa0]),  // select-5
        ((13, 0x00000040), &[0xb3, 0xc0]),  // select-6
        ((13, 0x00000080), &[0xd3, 0xe0]),  // select-7
        ((13, 0x00000100), &[0x01]),         // solo-0
        ((13, 0x00000200), &[0x14, 0x21]),  // solo-1
        ((13, 0x00000400), &[0x34, 0x41]),  // solo-2
        ((13, 0x00000800), &[0x54, 0x61]),  // solo-3
        ((13, 0x00001000), &[0x74, 0x81]),  // solo-4
        ((13, 0x00002000), &[0x94, 0xa1]),  // solo-5
        ((13, 0x00004000), &[0xb4, 0xc1]),  // solo-6
        ((13, 0x00008000), &[0xd4, 0xe1]),  // solo-7

        ((14, 0x00000001), &[0x02]),        // mute-0
        ((14, 0x00000002), &[0x15, 0x22]),  // mute-1
        ((14, 0x00000004), &[0x35, 0x42]),  // mute-2
        ((14, 0x00000008), &[0x55, 0x62]),  // mute-3
        ((14, 0x00000010), &[0x75, 0x82]),  // mute-4
        ((14, 0x00000020), &[0x95, 0xa2]),  // mute-5
        ((14, 0x00000040), &[0xb5, 0xc2]),  // mute-6
        ((14, 0x00000080), &[0xd5, 0xe2]),  // mute-7
    ];
}
