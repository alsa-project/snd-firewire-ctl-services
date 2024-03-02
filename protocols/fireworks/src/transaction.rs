// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2022 Takashi Sakamoto

//! Transaction protocol implementation defined by Echo Audio Digital Corporation for Fireworks
//! board module.
//!
//! The module includes transaction protocol implementation defined by Echo Audio Digital
//! Corporation for Fireworks board module. It has conflict with ALSA Fireworks driver.

use {
    glib::{
        subclass::{object::*, types::*},
        *,
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
        Object::new(&[]).expect("Failed to create EfwTransaction")
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
    use {super::*, once_cell::sync::Lazy};

    pub const COMMAND_OFFSET: u64 = 0xecc000000000;
    pub const RESPONSE_OFFSET: u64 = 0xecc080000000;
    pub const MAX_FRAME_SIZE: usize = 0x200;

    #[derive(Default)]
    pub struct EfwTransactionPrivate(RefCell<u32>, RefCell<Option<FwNode>>);

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

    impl ObjectImpl for EfwTransactionPrivate {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "node",
                    "node",
                    "An instance of FwNode",
                    FwNode::static_type(),
                    ParamFlags::READWRITE,
                )]
            });

            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "node" => self.1.borrow().as_ref().unwrap().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "node" => {
                    let node = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    *self.1.borrow_mut() = node;
                }
                _ => unimplemented!(),
            }
        }
    }

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
            if let Some(node) = self.1.borrow().as_ref() {
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
            if let Some(node) = self.1.borrow().as_ref() {
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
            let seqnum = *self.0.borrow();
            let next_seqnum = seqnum + 2;
            *self.0.borrow_mut() = if next_seqnum < (u16::MAX - 1) as u32 {
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
