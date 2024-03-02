// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocols defined for Tascam for FireWire series only with asynchronous communication.
//!
//! The module includes protocol implementation defined by Tascam for FireWire series only with
//! asynchronous communication.

pub mod fe8;

use {
    super::*,
    glib::{
        subclass::{object::*, types::*},
        *,
    },
    hinawa::{prelude::*, subclass::prelude::*, *},
    hitaki::{prelude::*, subclass::prelude::*, *},
    std::{cell::RefCell, sync::Mutex},
};

const ASYNCH_IMAGE_QUADLET_COUNT: usize = TascamExpander::QUADLET_COUNT;

glib::wrapper! {
    /// The implementation of `hitaki::TascamProtocol` so that it can cache status of hardware
    /// from message in asynchronous packet from the hardware as ALSA firewire-tascam driver does
    /// for message in isochronous packet.
    pub struct TascamExpander(ObjectSubclass<imp::TascamExpanderPrivate>)
        @extends FwResp, @implements TascamProtocol;
}

impl TascamExpander {
    pub const QUADLET_COUNT: usize = imp::RESPONSE_FRAME_SIZE / 4;

    pub fn new() -> Self {
        Object::new(&[]).expect("Failed to create TascamExpander")
    }

    fn node(&self) -> FwNode {
        self.property::<FwNode>("node")
    }

    pub fn bind(&self, node: &FwNode) -> Result<(), Error> {
        self.reserve_within_region(
            node,
            imp::RESPONSE_REGION_START,
            imp::RESPONSE_REGION_END,
            imp::RESPONSE_FRAME_SIZE as u32,
        )?;

        let mut addr = self.offset();
        addr |= (node.local_node_id() as u64) << 48;

        let mut req = FwReq::new();

        let mut addr_hi = ((addr >> 32) as u32).to_be_bytes();
        write_quadlet(&mut req, node, imp::ADDR_HIGH_OFFSET, &mut addr_hi, 100)?;

        let mut addr_lo = ((addr & 0xffffffff) as u32).to_be_bytes();
        write_quadlet(&mut req, node, imp::ADDR_LOW_OFFSET, &mut addr_lo, 100)?;

        self.set_property("node", node);

        Ok(())
    }

    pub fn listen(&self) -> Result<(), Error> {
        let mut frames = 1u32.to_be_bytes();
        let mut req = FwReq::new();
        write_quadlet(
            &mut req,
            &self.node(),
            imp::ENABLE_NOTIFICATION,
            &mut frames,
            100,
        )
    }

    pub fn unlisten(&self) {
        let mut frames = 0u32.to_be_bytes();
        let mut req = FwReq::new();
        let _ = write_quadlet(
            &mut req,
            &self.node(),
            imp::ENABLE_NOTIFICATION,
            &mut frames,
            100,
        );
    }

    pub fn unbind(&self) {
        self.release();

        let mut req = FwReq::new();
        let _ = write_quadlet(
            &mut req,
            &self.node(),
            imp::ADDR_HIGH_OFFSET,
            &mut [0; 4],
            100,
        );
        let _ = write_quadlet(
            &mut req,
            &self.node(),
            imp::ADDR_LOW_OFFSET,
            &mut [0; 4],
            100,
        );

        let private = imp::TascamExpanderPrivate::from_instance(self);
        *private.0.borrow_mut() = None;
    }
}

mod imp {
    use {super::*, once_cell::sync::Lazy};

    pub const RESPONSE_REGION_START: u64 = 0xffffe0000000;
    pub const RESPONSE_REGION_END: u64 = 0xfffff0000000;
    pub const RESPONSE_FRAME_SIZE: usize = 0x80;

    pub const ENABLE_NOTIFICATION: u64 = 0x0310;
    pub const ADDR_HIGH_OFFSET: u64 = 0x0314;
    pub const ADDR_LOW_OFFSET: u64 = 0x0318;

    #[derive(Default)]
    pub struct TascamExpanderPrivate(
        pub RefCell<Option<FwNode>>,
        RefCell<Mutex<[u32; super::TascamExpander::QUADLET_COUNT]>>,
    );

    #[glib::object_subclass]
    impl ObjectSubclass for TascamExpanderPrivate {
        const NAME: &'static str = "TascamExpander";
        type Type = super::TascamExpander;
        type ParentType = FwResp;
        type Interfaces = (TascamProtocol,);

        fn new() -> Self {
            Self::default()
        }
    }

    impl ObjectImpl for TascamExpanderPrivate {
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
                "node" => self.0.borrow().as_ref().unwrap().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _unit: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "node" => {
                    let node = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    *self.0.borrow_mut() = node;
                }
                _ => unimplemented!(),
            }
        }
    }

    fn parse_notification(
        image: &mut [u32],
        events: &mut Vec<(u32, u32, u32)>,
        tcode: FwTcode,
        frame: &[u8],
    ) -> FwRcode {
        if tcode == FwTcode::WriteQuadletRequest || tcode == FwTcode::WriteBlockRequest {
            let mut quadlet = [0; 4];
            (0..frame.len()).step_by(4).for_each(|pos| {
                quadlet.copy_from_slice(&frame[pos..(pos + 4)]);
                let value = u32::from_be_bytes(quadlet);
                let index = ((value & 0x00ff0000) >> 16) as usize;
                let state = value & 0x0000ffff;
                if index < image.len() && image[index] != state {
                    events.push((index as u32, image[index], state));
                    image[index] = state;
                }
            });
            FwRcode::Complete
        } else {
            FwRcode::TypeError
        }
    }

    impl FwRespImpl for TascamExpanderPrivate {
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
            if !resp.is_reserved() {
                return FwRcode::DataError;
            }

            let inst = match resp.downcast_ref::<super::TascamExpander>() {
                Some(inst) => inst,
                None => return FwRcode::DataError,
            };

            match self.0.borrow().as_ref() {
                Some(node) => {
                    if src != node.node_id() || offset != resp.offset() {
                        return FwRcode::AddressError;
                    }
                }
                None => return FwRcode::DataError,
            }

            let mut events = Vec::<(u32, u32, u32)>::new();

            let rcode = self
                .1
                .borrow_mut()
                .lock()
                .map(|mut image| {
                    parse_notification(&mut image[..], &mut events, tcode, frame);
                    FwRcode::Complete
                })
                .unwrap_or(FwRcode::DataError);

            events
                .iter()
                .for_each(|ev| inst.emit_changed(ev.0, ev.1, ev.2));

            rcode
        }
    }

    impl TascamProtocolImpl for TascamExpanderPrivate {
        fn read_state(&self, _unit: &Self::Type, state: &mut Vec<u32>) -> Result<(), Error> {
            if state.len() < super::TascamExpander::QUADLET_COUNT {
                let msg = format!(
                    "The size of buffer should be greater than 32 but {}",
                    state.len()
                );
                Err(Error::new(FileError::Inval, &msg))
            } else {
                self.1
                    .borrow()
                    .lock()
                    .map(|image| {
                        state.copy_from_slice(&image[..]);
                        state.truncate(super::TascamExpander::QUADLET_COUNT);
                    })
                    .map_err(|e| {
                        let msg = format!("{:?}", e);
                        Error::new(FileError::Io, &msg)
                    })
            }
        }

        fn changed(&self, _unit: &Self::Type, _index: u32, _before: u32, _after: u32) {}
    }
}
