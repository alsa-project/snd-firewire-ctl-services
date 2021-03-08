// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by RME GmbH for former models of Firewire series.

pub mod ff800;

use glib::Error;

use hinawa::{FwNode, FwTcode, FwReq, FwReqExtManual};

/// The structure to represent state of hardware meter.
///
/// Each value of 32 bit integer is between 0x00000000 and 0x7fffff00 to represent -90.03 and
/// 0.00 dB. When reaching saturation, 1 byte in LSB side represent ratio of overload.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct FormerMeterState{
    pub analog_inputs: Vec<i32>,
    pub spdif_inputs: Vec<i32>,
    pub adat_inputs: Vec<i32>,
    pub stream_inputs: Vec<i32>,
    pub analog_outputs: Vec<i32>,
    pub spdif_outputs: Vec<i32>,
    pub adat_outputs: Vec<i32>,
}

/// The trait to represent specification of hardware meter.
pub trait FormerMeterSpec {
    const ANALOG_INPUT_COUNT: usize;
    const SPDIF_INPUT_COUNT: usize;
    const ADAT_INPUT_COUNT: usize;
    const STREAM_INPUT_COUNT: usize;

    const ANALOG_OUTPUT_COUNT: usize;
    const SPDIF_OUTPUT_COUNT: usize;
    const ADAT_OUTPUT_COUNT: usize;

    fn create_meter_state() -> FormerMeterState {
        FormerMeterState{
            analog_inputs: vec![0;Self::ANALOG_INPUT_COUNT],
            spdif_inputs: vec![0;Self::SPDIF_INPUT_COUNT],
            adat_inputs: vec![0;Self::ADAT_INPUT_COUNT],
            stream_inputs: vec![0;Self::STREAM_INPUT_COUNT],
            analog_outputs: vec![0;Self::ANALOG_OUTPUT_COUNT],
            spdif_outputs: vec![0;Self::SPDIF_OUTPUT_COUNT],
            adat_outputs: vec![0;Self::ADAT_OUTPUT_COUNT],
        }
    }
}

/// The trait to represent meter protocol of Fireface 400.
pub trait RmeFfFormerMeterProtocol<T, U> : AsRef<FwReq>
    where T: AsRef<FwNode>,
          U: FormerMeterSpec + AsRef<FormerMeterState> + AsMut<FormerMeterState>,
{
    const METER_OFFSET: usize;

    fn read_meter(&self, node: &T, state: &mut U, timeout_ms: u32) -> Result<(), Error> {
        let phys_input_count = U::ANALOG_INPUT_COUNT + U::SPDIF_INPUT_COUNT + U::ADAT_INPUT_COUNT;
        let phys_output_count = U::ANALOG_OUTPUT_COUNT + U::SPDIF_OUTPUT_COUNT + U::ADAT_OUTPUT_COUNT;

        // NOTE:
        // Each of the first octuples is for level of corresponding source to mixer.
        // Each of the following octuples is for level of corresponding output from mixer (pre-fader).
        // Each of the following octuples is for level of corresponding output from mixer (post-fader).
        // Each of the following quadlets is for level of corresponding physical input.
        // Each of the following quadlets is for level of corresponding stream input.
        // Each of the following quadlets is for level of corresponding physical output.
        let length = 8 * (phys_input_count + phys_output_count * 2) +
                     4 * (phys_input_count + U::STREAM_INPUT_COUNT + phys_output_count);
        let mut raw = vec![0;length];
        self.as_ref().transaction_sync(node.as_ref(), FwTcode::ReadBlockRequest, Self::METER_OFFSET as u64,
                                       raw.len(), &mut raw, timeout_ms)
            .map(|_| {
                let s = state.as_mut();

                // TODO: pick up overload.
                let mut quadlet = [0;4];
                let mut offset = 8 * (phys_input_count + phys_output_count * 2);
                [
                    &mut s.analog_inputs[..],
                    &mut s.spdif_inputs[..],
                    &mut s.adat_inputs[..],
                    &mut s.stream_inputs[..],
                    &mut s.analog_outputs[..],
                    &mut s.spdif_outputs[..],
                    &mut s.adat_outputs[..],
                ].iter_mut()
                    .for_each(|meters| {
                        meters.iter_mut()
                            .for_each(|v| {
                                quadlet.copy_from_slice(&raw[offset..(offset + 4)]);
                                *v = i32::from_le_bytes(quadlet) & 0x7fffff00;
                                offset += 4;
                            });
                    });
            })
    }
}

/// The trait to represent output protocol specific to former models of RME Fireface.
///
/// The value for volume is between 0x00000000 and 0x00010000 through 0x00000001 and 0x00080000 to
/// represent the range from negative infinite to 6.00 dB through -90.30 dB and 0.00 dB.
pub trait RmeFormerOutputProtocol<T, U> : AsRef<FwReq>
    where T: AsRef<FwNode>,
          U: AsRef<[i32]> + AsMut<[i32]>,
{
    fn write_output_vol(&self, node: &T, ch: usize, vol: i32, timeout_ms: u32) -> Result<(), Error>;

    fn init_output_vols(&self, node: &T, state: &U, timeout_ms: u32) -> Result<(), Error> {
        state.as_ref().iter()
            .enumerate()
            .try_for_each(|(i, vol)| self.write_output_vol(node, i, *vol, timeout_ms))
    }

    fn write_output_vols(&self, node: &T, state: &mut U, vols: &[i32], timeout_ms: u32) -> Result<(), Error> {
        state.as_mut().iter_mut()
            .zip(vols.iter())
            .enumerate()
            .filter(|(_, (o, n))| !o.eq(n))
            .try_for_each(|(i, (o, n))| {
                self.write_output_vol(node, i, *n, timeout_ms)
                    .map(|_| *o = *n)
            })
    }
}
