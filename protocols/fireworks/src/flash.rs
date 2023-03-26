// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol about operations for on-board flash memory.
//!
//! The module includes protocol about operations for on-board flash memory defined by Echo Audio
//! Digital Corporation for Fireworks board module.

use super::*;

const CATEGORY_FLASH: u32 = 1;

const CMD_ERASE: u32 = 0;
const CMD_READ: u32 = 1;
const CMD_WRITE: u32 = 2;
const CMD_STATUS: u32 = 3;
const CMD_SESSION_BASE: u32 = 4;
const CMD_LOCK: u32 = 5;

/// The size of block in on-board flash memory in quadlet unit.
pub const BLOCK_QUADLET_COUNT: usize = 64;

/// The trait to express protocol about operations for on-board flash memory.
pub trait EfwFlashProtocol: EfwProtocolExtManual {
    /// The operation to erase block range (256 bytes) in on-board flash memory.
    fn flash_erase(&mut self, offset: u32, timeout_ms: u32) -> Result<(), Error> {
        let args = [offset];
        self.transaction(
            CATEGORY_FLASH,
            CMD_ERASE,
            &args,
            &mut Vec::new(),
            timeout_ms,
        )
    }

    /// The operation to read data from on-board flash memory.
    fn flash_read(&mut self, offset: u32, data: &mut [u32], timeout_ms: u32) -> Result<(), Error> {
        if data.len() > BLOCK_QUADLET_COUNT {
            let msg = format!(
                "The size of data should be less than {} but {}: ",
                BLOCK_QUADLET_COUNT,
                data.len()
            );
            Err(Error::new(FileError::Inval, &msg))?;
        }

        let args = [offset, data.len() as u32];
        let mut params = vec![0; 2 + BLOCK_QUADLET_COUNT];

        self.transaction(CATEGORY_FLASH, CMD_READ, &args, &mut params, timeout_ms)
            .map(|_| data.copy_from_slice(&params[2..(2 + data.len())]))
    }

    /// The operation to write data to on-board flash memory.
    fn flash_write(&mut self, offset: u32, data: &[u32], timeout_ms: u32) -> Result<(), Error> {
        if data.len() > BLOCK_QUADLET_COUNT {
            let msg = format!(
                "The size of data should be less than {} but {}: ",
                BLOCK_QUADLET_COUNT,
                data.len()
            );
            Err(Error::new(FileError::Inval, &msg))?;
        }

        let mut args = vec![0; 2 + BLOCK_QUADLET_COUNT];
        args[0] = offset;
        args[1] = data.len() as u32;
        args[2..(2 + data.len())].copy_from_slice(&data);

        self.transaction(
            CATEGORY_FLASH,
            CMD_WRITE,
            &args,
            &mut Vec::new(),
            timeout_ms,
        )
    }

    /// The operation to get status of on-board flash memory.
    fn flash_is_locked(&mut self, timeout_ms: u32) -> Result<bool, Error> {
        if let Err(e) =
            self.transaction(CATEGORY_FLASH, CMD_STATUS, &[], &mut Vec::new(), timeout_ms)
        {
            if e.kind::<EfwProtocolError>() == Some(EfwProtocolError::FlashBusy) {
                // It's locked.
                Ok(true)
            } else {
                Err(e)
            }
        } else {
            // It's unlocked.
            Ok(false)
        }
    }

    /// The operation to get offset for base of session in on-board flash memory.
    fn flash_session_base(&mut self, timeout_ms: u32) -> Result<u32, Error> {
        let mut params = vec![0];

        self.transaction(
            CATEGORY_FLASH,
            CMD_SESSION_BASE,
            &[],
            &mut params,
            timeout_ms,
        )
        .map(|_| params[0])
    }

    /// The operation to lock on-board flash memory, required for FPGA models only.
    fn flash_lock(&mut self, locking: bool, timeout_ms: u32) -> Result<(), Error> {
        let args = vec![locking as u32];
        self.transaction(CATEGORY_FLASH, CMD_LOCK, &args, &mut Vec::new(), timeout_ms)
    }
}

impl<O: EfwProtocolExtManual> EfwFlashProtocol for O {}

#[cfg(test)]
mod test {
    use super::*;
    use glib::{translate::FromGlib, SignalHandlerId};
    use std::cell::RefCell;

    const BLOCK_SIZE: usize = 4 * BLOCK_QUADLET_COUNT as usize;
    const TIMEOUT: u32 = 10;

    #[derive(Default)]
    struct TestProtocol(RefCell<StateMachine>);

    #[test]
    fn flash_lock_test() {
        let mut proto = TestProtocol::default();

        // The initial status should be locked.
        assert_eq!(proto.flash_is_locked(TIMEOUT), Ok(true));

        // The erase operation should be failed due to locked status.
        let err = proto.flash_erase(256, TIMEOUT);
        if let Err(e) = err {
            assert_eq!(
                e.kind::<EfwProtocolError>(),
                Some(EfwProtocolError::FlashBusy)
            );
        } else {
            unreachable!();
        }

        // The write operation should be failed as well due to locked status.
        let data = [0; 16];
        let err = proto.flash_write(256, &data, TIMEOUT);
        if let Err(e) = err {
            assert_eq!(
                e.kind::<EfwProtocolError>(),
                Some(EfwProtocolError::FlashBusy)
            );
        } else {
            unreachable!();
        }

        // The read operation is always available.
        let mut data = [0; 64];
        proto.flash_read(0, &mut data, TIMEOUT).unwrap();

        // Unlock it.
        proto.flash_lock(false, TIMEOUT).unwrap();
        assert_eq!(proto.flash_is_locked(TIMEOUT), Ok(false));

        // The erase operation should be available now.
        proto.flash_erase(256, TIMEOUT).unwrap();

        // The write operation should be available now.
        let data = [0; 16];
        proto.flash_write(256, &data, TIMEOUT).unwrap();

        // The read operation is always available.
        let mut data = [0; 64];
        proto.flash_read(0, &mut data, TIMEOUT).unwrap();

        // Lock it.
        proto.flash_lock(true, TIMEOUT).unwrap();
        assert_eq!(proto.flash_is_locked(TIMEOUT), Ok(true));

        // The erase operation should be failed again;
        let err = proto.flash_erase(256, TIMEOUT);
        if let Err(e) = err {
            assert_eq!(
                e.kind::<EfwProtocolError>(),
                Some(EfwProtocolError::FlashBusy)
            );
        } else {
            unreachable!();
        }

        // The write operation should be failed as well;
        let data = [0; 16];
        let err = proto.flash_write(256, &data, TIMEOUT);
        if let Err(e) = err {
            assert_eq!(
                e.kind::<EfwProtocolError>(),
                Some(EfwProtocolError::FlashBusy)
            );
        } else {
            unreachable!();
        }

        // The read operation is always available.
        let mut data = [0; 64];
        proto.flash_read(0, &mut data, TIMEOUT).unwrap();
    }

    #[test]
    fn flash_update_test() {
        let mut proto = TestProtocol::default();

        let count = proto.0.borrow().memory.len() / 4;
        (0..count).for_each(|i| {
            let pos = i * 4;
            proto.0.borrow_mut().memory[pos..(pos + 4)].copy_from_slice(&(i as u32).to_be_bytes());
        });

        proto.flash_lock(false, TIMEOUT).unwrap();
        proto.flash_erase(256, TIMEOUT).unwrap();

        // Check near the boundary between first and second blocks.
        let mut data = [0; 16];
        proto.flash_read(248, &mut data, TIMEOUT).unwrap();
        assert_eq!(&data, &[62, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        // Check near the boundary between second and third blocks.
        let mut data = [0; 8];
        proto.flash_read(504, &mut data, TIMEOUT).unwrap();
        assert_eq!(&data, &[0, 0, 128, 129, 130, 131, 132, 133]);

        // Update the second block.
        let mut data = [0; BLOCK_QUADLET_COUNT];
        data.iter_mut()
            .enumerate()
            .for_each(|(i, q)| *q = u32::MAX - i as u32);
        proto.flash_write(256, &data, TIMEOUT).unwrap();

        // Check near the boundary between second and third block.
        let mut data = [0; 6];
        proto.flash_read(504, &mut data, TIMEOUT).unwrap();
        assert_eq!(&data, &[4294967233, 4294967232, 128, 129, 130, 131]);
    }

    struct StateMachine {
        // Here, the state machine is defined to have four blocks in which the first block is
        // immutable.
        memory: [u8; 4 * BLOCK_SIZE],
        // At initial state, the memory is locked against erase and write operation.
        locked: bool,
    }

    impl Default for StateMachine {
        fn default() -> Self {
            Self {
                memory: [0; 4 * BLOCK_SIZE],
                locked: true,
            }
        }
    }

    impl StateMachine {
        fn erase_block(&mut self, args: &[u32], params: &mut Vec<u32>) -> Result<(), Error> {
            if params.len() > 0 {
                Err(Error::new(
                    EfwProtocolError::BadParameter,
                    "Useless parameter is given",
                ))
            } else if args.len() < 1 {
                Err(Error::new(
                    EfwProtocolError::BadCommand,
                    "Argument is shorter than expected",
                ))
            } else {
                // Align to block.
                let pos = (args[0] as usize) / BLOCK_SIZE * BLOCK_SIZE;
                if pos == 0 {
                    Err(Error::new(
                        EfwProtocolError::BadCommand,
                        "The first block is immutable",
                    ))
                } else if pos > self.memory.len() {
                    Err(Error::new(
                        EfwProtocolError::BadCommand,
                        "The offset is out of range",
                    ))
                } else if self.locked {
                    Err(Error::new(
                        EfwProtocolError::FlashBusy,
                        "The flash memory is locked",
                    ))
                } else {
                    self.memory[pos..(pos + BLOCK_SIZE)].fill(0);
                    Ok(())
                }
            }
        }

        fn read_data(&self, args: &[u32], params: &mut Vec<u32>) -> Result<(), Error> {
            if args.len() < 2 {
                Err(Error::new(
                    EfwProtocolError::BadCommand,
                    "Argument is shorter than expected",
                ))
            } else {
                let offset = args[0] as usize;
                let count = args[1] as usize;
                if count >= BLOCK_SIZE {
                    let msg = "The count of data should be less than size of block";
                    Err(Error::new(EfwProtocolError::BadCommand, &msg))
                } else if offset + 4 * count > self.memory.len() {
                    Err(Error::new(
                        EfwProtocolError::BadCommand,
                        "The offset plus count is out of range",
                    ))
                } else {
                    if params.len() < 2 + count {
                        Err(Error::new(
                            EfwProtocolError::BadParameter,
                            "Parameter is shorter than expected",
                        ))
                    } else {
                        params[0] = offset as u32;
                        params[1] = count as u32;

                        let mut quadlet = [0; 4];
                        params[2..].iter_mut().enumerate().for_each(|(i, d)| {
                            let pos = offset as usize + i * 4;
                            quadlet.copy_from_slice(&self.memory[pos..(pos + 4)]);
                            *d = u32::from_be_bytes(quadlet);
                        });
                        Ok(())
                    }
                }
            }
        }

        fn write_data(&mut self, args: &[u32], params: &mut Vec<u32>) -> Result<(), Error> {
            if params.len() > 0 {
                Err(Error::new(
                    EfwProtocolError::BadParameter,
                    "Useless parameter is given",
                ))
            } else if args.len() < 3 {
                Err(Error::new(
                    EfwProtocolError::BadCommand,
                    "Argument is shorter than expected",
                ))
            } else {
                let offset = args[0] as usize;
                if offset < BLOCK_SIZE {
                    Err(Error::new(
                        EfwProtocolError::BadCommand,
                        "The first block is immutable",
                    ))
                } else {
                    let count = args[1] as usize;
                    let data = &args[2..];

                    if data.len() < count {
                        Err(Error::new(
                            EfwProtocolError::BadCommand,
                            "Contradiction between count and data",
                        ))
                    } else if data.len() > BLOCK_QUADLET_COUNT {
                        let msg = "The count of data should be less than size of block";
                        Err(Error::new(EfwProtocolError::BadCommand, msg))
                    } else if offset + 4 * data.len() > self.memory.len() {
                        Err(Error::new(
                            EfwProtocolError::BadCommand,
                            "The offset plus length is out of range",
                        ))
                    } else if self.locked {
                        Err(Error::new(
                            EfwProtocolError::FlashBusy,
                            "The flash memory is locked",
                        ))
                    } else {
                        data.iter().enumerate().for_each(|(i, d)| {
                            let pos = offset + i * 4;
                            self.memory[pos..(pos + 4)].copy_from_slice(&d.to_be_bytes());
                        });
                        Ok(())
                    }
                }
            }
        }

        fn get_status(&self, args: &[u32], params: &mut Vec<u32>) -> Result<(), Error> {
            if args.len() > 0 {
                Err(Error::new(
                    EfwProtocolError::BadCommand,
                    "Useless argument is given",
                ))
            } else if params.len() > 0 {
                Err(Error::new(
                    EfwProtocolError::BadParameter,
                    "Useless parameter is given",
                ))
            } else if self.locked {
                Err(Error::new(
                    EfwProtocolError::FlashBusy,
                    "The flash memory is locked",
                ))
            } else {
                Ok(())
            }
        }

        fn get_session_base(&self, args: &[u32], params: &mut Vec<u32>) -> Result<(), Error> {
            if args.len() > 0 {
                Err(Error::new(
                    EfwProtocolError::BadCommand,
                    "Useless argument is given",
                ))
            } else if params.len() < 1 {
                Err(Error::new(
                    EfwProtocolError::BadParameter,
                    "Parameter is shorter than expected",
                ))
            } else {
                params[0] = BLOCK_SIZE as u32;
                Ok(())
            }
        }

        fn lock_memory(&mut self, args: &[u32], params: &mut Vec<u32>) -> Result<(), Error> {
            if params.len() > 0 {
                Err(Error::new(
                    EfwProtocolError::BadCommand,
                    "Useless parameter is given",
                ))
            } else if args.len() < 1 {
                Err(Error::new(
                    EfwProtocolError::BadParameter,
                    "Argument is shorter than expected",
                ))
            } else {
                self.locked = args[0] > 0;
                Ok(())
            }
        }
    }

    impl EfwProtocolExtManual for TestProtocol {
        fn transaction(
            &self,
            category: u32,
            command: u32,
            args: &[u32],
            params: &mut Vec<u32>,
            _: u32,
        ) -> Result<(), glib::Error> {
            assert_eq!(category, CATEGORY_FLASH);
            match command {
                CMD_ERASE => self.0.borrow_mut().erase_block(args, params),
                CMD_READ => self.0.borrow_mut().read_data(args, params),
                CMD_WRITE => self.0.borrow_mut().write_data(args, params),
                CMD_STATUS => self.0.borrow_mut().get_status(args, params),
                CMD_SESSION_BASE => self.0.borrow_mut().get_session_base(args, params),
                CMD_LOCK => self.0.borrow_mut().lock_memory(args, params),
                _ => unreachable!(),
            }
        }

        fn emit_responded(&self, _: u32, _: u32, _: u32, _: u32, _: EfwProtocolError, _: &[u32]) {
            // Omitted.
        }

        fn connect_responded<F>(&self, _f: F) -> SignalHandlerId
        where
            F: Fn(&Self, u32, u32, u32, u32, EfwProtocolError, &[u32]) + 'static,
        {
            // Dummy.
            unsafe { SignalHandlerId::from_glib(0) }
        }
    }
}
