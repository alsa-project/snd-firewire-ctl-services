// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by RME GmbH for Fireface 802.

use hinawa::FwReq;

/// The structure to represent unique protocol for Fireface 802.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Ff802Protocol(FwReq);

impl AsRef<FwReq> for Ff802Protocol {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}
