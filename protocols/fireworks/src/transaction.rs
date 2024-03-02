// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2022 Takashi Sakamoto

//! Transaction protocol implementation defined by Echo Audio Digital Corporation for Fireworks
//! board module.
//!
//! The module includes transaction protocol implementation defined by Echo Audio Digital
//! Corporation for Fireworks board module. It has conflict with ALSA Fireworks driver.

use {
    glib::{
        subclass::{object::*, prelude::*},
        Properties, *,
    },
    hinawa::{prelude::*, subclass::prelude::*, *},
    hitaki::{prelude::*, subclass::prelude::*, *},
    std::cell::RefCell,
};

glib::wrapper! {
    /// The implementation of hitaki`::EfwProtocolExt` and `hitaki::EfwProtocolExtManual` traits
    /// so that it can perform Echo Audio Fireworks transaction instead of ALSA Fireworks driver.
    pub struct EfwTransaction(ObjectSubclass<imp::EfwTransactionPrivate>)
        @extends FwResp, @implements EfwProtocol;
}

impl EfwTransaction {
    pub fn new() -> Self {
        object::Object::new()
    }

    pub fn bind(&self, node: &FwNode) -> Result<(), Error> {
        self.reserve_within_region(
            node,
            imp::RESPONSE_OFFSET,
            imp::RESPONSE_OFFSET + (imp::MAX_FRAME_SIZE as u64),
            imp::MAX_FRAME_SIZE as u32,
        )
        .map(|_| self.set_property("node", node))
    }

    pub fn unbind(&self) {
        self.release();
    }
}

mod imp {
    use super::*;

    pub const COMMAND_OFFSET: u64 = 0xecc000000000;
    pub const RESPONSE_OFFSET: u64 = 0xecc080000000;
    pub const MAX_FRAME_SIZE: usize = 0x200;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::EfwTransaction)]
    pub struct EfwTransactionPrivate {
        #[property(set)]
        node_id: RefCell<u32>,
        #[property(set)]
        seqnum: RefCell<Option<FwNode>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EfwTransactionPrivate {
        const NAME: &'static str = "EfwTransaction";
        type Type = super::EfwTransaction;
        type ParentType = FwResp;
        type Interfaces = (EfwProtocol,);

        fn new() -> Self {
            Self::default()
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for EfwTransactionPrivate {}

    impl FwRespImpl for EfwTransactionPrivate {
        fn requested(
            &self,
            resp: &Self::Type,
            tcode: FwTcode,
            offset: u64,
            src: u32,
            _dst: u32,
            _card: u32,
            _generation: u32,
            _tstamp: u32,
            frame: &[u8],
        ) -> FwRcode {
            if let Some(node) = self.seqnum.borrow().as_ref() {
                if tcode != FwTcode::WriteBlockRequest {
                    FwRcode::TypeError
                } else if src != node.node_id() || offset != RESPONSE_OFFSET {
                    FwRcode::AddressError
                } else if !resp.is_reserved() || frame.len() < 24 {
                    FwRcode::DataError
                } else {
                    let inst = resp.downcast_ref::<super::EfwTransaction>().unwrap();
                    inst.receive_response(frame);

                    FwRcode::Complete
                }
            } else {
                // Not prepared.
                FwRcode::DataError
            }
        }
    }

    impl EfwProtocolImpl for EfwTransactionPrivate {
        fn transmit_request(&self, _: &Self::Type, buffer: &[u8]) -> Result<(), Error> {
            if let Some(node) = self.seqnum.borrow().as_ref() {
                let req = FwReq::new();
                let mut frame = buffer.to_owned();
                req.transaction(
                    node,
                    FwTcode::WriteBlockRequest,
                    COMMAND_OFFSET,
                    frame.len(),
                    &mut frame,
                    100,
                )?;

                Ok(())
            } else {
                Err(Error::new(EfwProtocolError::Bad, "Not prepared."))
            }
        }

        fn get_seqnum(&self, _: &Self::Type) -> u32 {
            let seqnum = *self.node_id.borrow();
            let next_seqnum = seqnum + 2;
            *self.node_id.borrow_mut() = if next_seqnum < (u16::MAX - 1) as u32 {
                next_seqnum
            } else {
                0
            };
            seqnum
        }

        fn responded(
            &self,
            _unit: &Self::Type,
            _version: u32,
            _seqnum: u32,
            _category: u32,
            _command: u32,
            _status: EfwProtocolError,
            _params: &[u32],
        ) {
        }
    }
}
