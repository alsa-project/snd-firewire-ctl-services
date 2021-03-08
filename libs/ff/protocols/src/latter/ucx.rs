// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by RME GmbH for Fireface UCX.

use hinawa::FwReq;

/// The structure to represent unique protocol for Fireface UCX.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct FfUcxProtocol(FwReq);

impl AsRef<FwReq> for FfUcxProtocol {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}
