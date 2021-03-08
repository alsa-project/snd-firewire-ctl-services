// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by RME GmbH for Fireface 400.

use hinawa::FwReq;

/// The structure to represent unique protocol for Fireface 400.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Ff400Protocol(FwReq);

impl AsRef<FwReq> for Ff400Protocol {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}
