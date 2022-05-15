// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2022 Takashi Sakamoto

//! Transaction protocol implementation defined by Echo Audio Digital Corporation for Fireworks
//! board module.
//!
//! The module includes transaction protocol implementation defined by Echo Audio Digital
//! Corporation for Fireworks board module. It has conflict with ALSA Fireworks driver.

use {
    glib::{
        subclass::{object::*, simple, types::*},
        translate::*,
        *,
    },
    hinawa::{subclass::fw_resp::*, *},
    hitaki::{subclass::efw_protocol::*, *},
    std::cell::RefCell,
};

glib_wrapper! {
    pub struct EfwTransaction(
        Object<simple::InstanceStruct<imp::EfwTransactionPrivate>,
        simple::ClassStruct<imp::EfwTransactionPrivate>, EfwTransactionClass>
    ) @extends FwResp, @implements EfwProtocol;

    match fn {
        get_type => || imp::EfwTransactionPrivate::get_type().to_glib(),
    }
}

impl EfwTransaction {
    pub fn new() -> Self {
        Object::new(Self::static_type(), &[])
            .expect("Failed to create EfwTransaction")
            .downcast()
            .expect("Created row data is of wrong type")
    }

    pub fn bind(&self, node: &FwNode) -> Result<(), Error> {
        self.reserve_within_region(
            node,
            imp::RESPONSE_OFFSET,
            imp::RESPONSE_OFFSET + (imp::MAX_FRAME_SIZE as u64),
            imp::MAX_FRAME_SIZE as u32,
        )
        .map(|_| self.set_property("node", node).unwrap())
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

    #[derive(Default)]
    pub struct EfwTransactionPrivate(RefCell<u32>, RefCell<Option<FwNode>>);

    static PROPERTIES: [Property; 1] = [Property("node", |node| {
        ParamSpec::object(
            node,
            "node",
            "An instance of FwNode",
            FwNode::static_type(),
            ParamFlags::READWRITE,
        )
    })];

    impl ObjectSubclass for EfwTransactionPrivate {
        const NAME: &'static str = "EfwTransaction";
        type ParentType = FwResp;
        type Instance = simple::InstanceStruct<Self>;
        type Class = simple::ClassStruct<Self>;

        glib_object_subclass!();

        fn new() -> Self {
            Self::default()
        }

        fn class_init(klass: &mut Self::Class) {
            klass.install_properties(&PROPERTIES);
        }

        fn type_init(type_: &mut InitializingType<Self>) {
            type_.add_interface::<EfwProtocol>();
        }
    }

    impl ObjectImpl for EfwTransactionPrivate {
        glib_object_impl!();

        fn get_property(&self, _obj: &Object, id: usize) -> Result<Value, ()> {
            let prop = &PROPERTIES[id];

            match *prop {
                Property("node", ..) => Ok(self.1.borrow().as_ref().unwrap().to_value()),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _obj: &Object, id: usize, value: &Value) {
            let prop = &PROPERTIES[id];

            match *prop {
                Property("node", ..) => {
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
        fn requested2(
            &self,
            resp: &FwResp,
            tcode: FwTcode,
            offset: u64,
            src: u32,
            _dst: u32,
            _card: u32,
            _generation: u32,
            frame: &[u8],
        ) -> FwRcode {
            if let Some(node) = self.1.borrow().as_ref() {
                if tcode != FwTcode::WriteBlockRequest {
                    FwRcode::TypeError
                } else if src != node.get_property_node_id() || offset != RESPONSE_OFFSET {
                    FwRcode::AddressError
                } else if !resp.get_property_is_reserved() || frame.len() < 24 {
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
        fn transmit_request(&self, _: &EfwProtocol, buffer: &[u8]) -> Result<(), Error> {
            if let Some(node) = self.1.borrow().as_ref() {
                let req = FwReq::new();
                let mut frame = buffer.to_owned();
                req.transaction_sync(
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

        fn get_seqnum(&self, _: &EfwProtocol) -> u32 {
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
            _unit: &EfwProtocol,
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
