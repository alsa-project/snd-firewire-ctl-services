// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2024 Takashi Sakamoto

//! Protocol specific to Weiss Engineering normal models.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Weiss Engineering.

use super::*;

/// Protocol implementation specific to ADC2.
#[derive(Default, Debug)]
pub struct WeissAdc2Protocol;

impl TcatOperation for WeissAdc2Protocol {}

// clock caps: 44100 48000 88200 96000 176400 192000 aes1
// clock source names: AES12\Unused\Unused\Unused\Unused\Unused\Unused\Unused\Unused\Unused\Unused\Unused\Unused\\
impl TcatGlobalSectionSpecification for WeissAdc2Protocol {}

/// Protocol implementation specific to Vesta.
#[derive(Default, Debug)]
pub struct WeissVestaProtocol;

impl TcatOperation for WeissVestaProtocol {}

// clock caps: 44100 48000 88200 96000 176400 192000 aes1 aes2 aes3 arx1 internal
// clock source names: AES/EBU (XLR)\S/PDIF (RCA)\S/PDIF (TOSLINK)\Unused\Unused\Unused\Unused\Unused\Unused\Unused\Unused\Unused\Internal\\
impl TcatGlobalSectionSpecification for WeissVestaProtocol {}

/// Protocol implementation specific to DAC2/Minerva.
#[derive(Default, Debug)]
pub struct WeissDac2Protocol;

impl TcatOperation for WeissDac2Protocol {}

// clock caps: 44100 48000 88200 96000 176400 192000 aes1 aes2 aes3 arx1 internal
// clock source names: AES/EBU (XLR)\S/PDIF (RCA)\S/PDIF (TOSLINK)\Unused\Unused\Unused\Unused\Unused\Unused\Unused\Unused\Unused\Internal\\
impl TcatGlobalSectionSpecification for WeissDac2Protocol {}

/// Protocol implementation specific to AFI1.
#[derive(Default, Debug)]
pub struct WeissAfi1Protocol;

impl TcatOperation for WeissAfi1Protocol {}

// clock caps: 44100 48000 88200 96000 176400 192000 aes1 aes2 aes3 aes4 adat wc internal
// clock source names: AES12\AES34\AES56\AES78\Unused\ADAT\Unused\Word Clock\Unused\Unused\Unused\Unused\Internal\\
impl TcatGlobalSectionSpecification for WeissAfi1Protocol {}

/// Protocol implementation specific to DAC202 and Maya Edition.
#[derive(Default, Debug)]
pub struct WeissDac202Protocol;

impl TcatOperation for WeissDac202Protocol {}

// clock caps: 44100 48000 88200 96000 176400 192000 aes1 aes2 aes3 wc arx1 internal
// clock source names: AES/EBU (XLR)\S/PDIF (RCA)\S/PDIF (TOSLINK)\Unused\Unused\Unused\Unused\Word Clock\Unused\Unused\Unused\Unused\Internal\\
impl TcatGlobalSectionSpecification for WeissDac202Protocol {}

/// Protocol implementation specific to INT202, INT203, and FireWire option card for DAC1.
#[derive(Default, Debug)]
pub struct WeissInt203Protocol;

impl TcatOperation for WeissInt203Protocol {}

// clock caps: 44100 48000 88200 96000 176400 192000 aes1 aes2 arx1 internal
// clock source names: AES/EBU (XLR)\S/PDIF (RCA)\Unused\Unused\Unused\Unused\Unused\Unused\Unused\Unused\Unused\Unused\Internal\\
impl TcatGlobalSectionSpecification for WeissInt203Protocol {}
