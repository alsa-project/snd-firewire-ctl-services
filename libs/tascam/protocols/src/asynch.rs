// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocols defined for Tascam for FireWire series only with asynchronous communication.
//!
//! The module includes protocol implementation defined by Tascam for FireWire series only with
//! asynchronous communication.

pub mod fe8;

use {
    super::*,
    glib::{
        subclass::{object::*, simple, types::*},
        translate::*,
        *,
    },
    hinawa::{subclass::fw_resp::*, *},
    hitaki::{subclass::tascam_protocol::*, *},
    std::{cell::RefCell, sync::Mutex},
};

glib_wrapper! {
    pub struct TascamExpander(
        Object<simple::InstanceStruct<imp::TascamExpanderPrivate>,
        simple::ClassStruct<imp::TascamExpanderPrivate>, TascamExpanderClass>
    ) @extends FwResp, @implements TascamProtocol;

    match fn {
        get_type => || imp::TascamExpanderPrivate::get_type().to_glib(),
    }
}

impl TascamExpander {
    pub const QUADLET_COUNT: usize = imp::RESPONSE_FRAME_SIZE / 4;

    pub fn new() -> Self {
        Object::new(Self::static_type(), &[])
            .expect("Failed to create TascamExpander")
            .downcast()
            .expect("Created row data is of wrong type")
    }

    fn get_property_node(&self) -> Result<FwNode, Error> {
        self.get_property("node")
            .map_err(|e| Error::new(FileError::Nxio, &e.message))?
            .get::<FwNode>()
            .map_err(|e| Error::new(FileError::Nxio, &format!("{:?}", e)))?
            .ok_or_else(|| Error::new(FileError::Nxio, "node property is not available"))
    }

    pub fn bind(&self, node: &FwNode) -> Result<(), Error> {
        self.reserve_within_region(
            node,
            imp::RESPONSE_REGION_START,
            imp::RESPONSE_REGION_END,
            imp::RESPONSE_FRAME_SIZE as u32,
        )?;

        let mut addr = self.get_property_offset();
        addr |= (node.get_property_local_node_id() as u64) << 48;

        let mut req = FwReq::new();

        let mut addr_hi = ((addr >> 32) as u32).to_be_bytes();
        write_quadlet(&mut req, node, imp::ADDR_HIGH_OFFSET, &mut addr_hi, 100)?;

        let mut addr_lo = ((addr & 0xffffffff) as u32).to_be_bytes();
        write_quadlet(&mut req, node, imp::ADDR_LOW_OFFSET, &mut addr_lo, 100)?;

        self.set_property("node", node).unwrap();

        Ok(())
    }

    pub fn listen(&self) -> Result<(), Error> {
        let node = self.get_property_node()?;
        let mut frames = 1u32.to_be_bytes();
        let mut req = FwReq::new();
        write_quadlet(&mut req, &node, imp::ENABLE_NOTIFICATION, &mut frames, 100)
    }

    pub fn unlisten(&self) {
        if let Ok(node) = self.get_property_node() {
            let mut frames = 0u32.to_be_bytes();
            let mut req = FwReq::new();
            let _ = write_quadlet(&mut req, &node, imp::ENABLE_NOTIFICATION, &mut frames, 100);
        }
    }

    pub fn unbind(&self) {
        self.release();

        if let Ok(node) = self.get_property_node() {
            let mut req = FwReq::new();
            let _ = write_quadlet(&mut req, &node, imp::ADDR_HIGH_OFFSET, &mut [0; 4], 100);
            let _ = write_quadlet(&mut req, &node, imp::ADDR_LOW_OFFSET, &mut [0; 4], 100);
        }

        let private = imp::TascamExpanderPrivate::from_instance(self);
        *private.0.borrow_mut() = None;
    }
}

mod imp {
    use super::*;

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

    static PROPERTIES: [Property; 1] = [Property("node", |node| {
        ParamSpec::object(
            node,
            "node",
            "An instance of FwNode",
            FwNode::static_type(),
            ParamFlags::READWRITE,
        )
    })];

    impl ObjectSubclass for TascamExpanderPrivate {
        const NAME: &'static str = "TascamExpander";
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
            type_.add_interface::<TascamProtocol>();
        }
    }

    impl ObjectImpl for TascamExpanderPrivate {
        glib_object_impl!();

        fn get_property(&self, _obj: &Object, id: usize) -> Result<Value, ()> {
            let prop = &PROPERTIES[id];

            match *prop {
                Property("node", ..) => Ok(self.0.borrow().as_ref().unwrap().to_value()),
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
            if !resp.get_property_is_reserved() {
                return FwRcode::DataError;
            }

            let inst = match resp.downcast_ref::<super::TascamExpander>() {
                Some(inst) => inst,
                None => return FwRcode::DataError,
            };

            match self.0.borrow().as_ref() {
                Some(node) => {
                    if src != node.get_property_node_id() || offset != resp.get_property_offset() {
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
        fn read_state(&self, _unit: &TascamProtocol, state: &mut Vec<u32>) -> Result<(), Error> {
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

        fn changed(&self, _unit: &TascamProtocol, _index: u32, _before: u32, _after: u32) {}
    }
}
