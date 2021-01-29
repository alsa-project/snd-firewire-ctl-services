// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use hinawa::FwReq;

#[derive(Default, Debug)]
pub struct FStudioProto(FwReq);

impl AsRef<FwReq> for FStudioProto {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}
