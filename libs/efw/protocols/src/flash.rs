// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol about operations for on-board flash memory.
//!
//! The crate includes protocol about operations for on-board flash memory defined by Echo Audio
//! Digital Corporation for Fireworks board module.

use glib::{Error, FileError};

use hinawa::SndEfwStatus;

use super::EfwProtocol;

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
pub trait EfwFlashProtocol: EfwProtocol {
    /// The operation to erase block range (256 bytes) in on-board flash memory.
    fn flash_erase(&mut self, offset: u32, timeout_ms: u32) -> Result<(), Error> {
        let args = [offset];
        self.transaction_sync(CATEGORY_FLASH, CMD_ERASE, Some(&args), None, timeout_ms)
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

        self.transaction_sync(
            CATEGORY_FLASH,
            CMD_READ,
            Some(&args),
            Some(&mut params),
            timeout_ms,
        )
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

        self.transaction_sync(CATEGORY_FLASH, CMD_WRITE, Some(&args), None, timeout_ms)
    }

    /// The operation to get status of on-board flash memory.
    fn flash_is_locked(&mut self, timeout_ms: u32) -> Result<bool, Error> {
        if let Err(e) = self.transaction_sync(CATEGORY_FLASH, CMD_STATUS, None, None, timeout_ms) {
            if e.kind::<SndEfwStatus>() == Some(SndEfwStatus::FlashBusy) {
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

        self.transaction_sync(
            CATEGORY_FLASH,
            CMD_SESSION_BASE,
            None,
            Some(&mut params),
            timeout_ms,
        )
        .map(|_| params[0])
    }

    /// The operation to lock on-board flash memory, required for FPGA models only.
    fn flash_lock(&mut self, locking: bool, timeout_ms: u32) -> Result<(), Error> {
        let args = vec![locking as u32];
        self.transaction_sync(CATEGORY_FLASH, CMD_LOCK, Some(&args), None, timeout_ms)
    }
}

impl<O: EfwProtocol> EfwFlashProtocol for O {}

#[cfg(test)]
mod test {
    use super::*;

    use hinawa::SndEfwStatus;

    const BLOCK_SIZE: usize = 4 * BLOCK_QUADLET_COUNT as usize;
    const TIMEOUT: u32 = 10;

    #[derive(Default)]
    struct TestProtocol(StateMachine);

    #[test]
    fn flash_lock_test() {
        let mut proto = TestProtocol::default();

        // The initial status should be locked.
        assert_eq!(proto.flash_is_locked(TIMEOUT), Ok(true));

        // The erase operation should be failed due to locked status.
        let err = proto.flash_erase(256, TIMEOUT);
        if let Err(e) = err {
            assert_eq!(e.kind::<SndEfwStatus>(), Some(SndEfwStatus::FlashBusy));
        } else {
            unreachable!();
        }

        // The write operation should be failed as well due to locked status.
        let data = [0;16];
        let err = proto.flash_write(256, &data, TIMEOUT);
        if let Err(e) = err {
            assert_eq!(e.kind::<SndEfwStatus>(), Some(SndEfwStatus::FlashBusy));
        } else {
            unreachable!();
        }

        // The read operation is always available.
        let mut data = [0;64];
        proto.flash_read(0, &mut data, TIMEOUT).unwrap();

        // Unlock it.
        proto.flash_lock(false, TIMEOUT).unwrap();
        assert_eq!(proto.flash_is_locked(TIMEOUT), Ok(false));

        // The erase operation should be available now.
        proto.flash_erase(256, TIMEOUT).unwrap();

        // The write operation should be available now.
        let data = [0;16];
        proto.flash_write(256, &data, TIMEOUT).unwrap();

        // The read operation is always available.
        let mut data = [0;64];
        proto.flash_read(0, &mut data, TIMEOUT).unwrap();

        // Lock it.
        proto.flash_lock(true, TIMEOUT).unwrap();
        assert_eq!(proto.flash_is_locked(TIMEOUT), Ok(true));

        // The erase operation should be failed again;
        let err = proto.flash_erase(256, TIMEOUT);
        if let Err(e) = err {
            assert_eq!(e.kind::<SndEfwStatus>(), Some(SndEfwStatus::FlashBusy));
        } else {
            unreachable!();
        }

        // The write operation should be failed as well;
        let data = [0;16];
        let err = proto.flash_write(256, &data, TIMEOUT);
        if let Err(e) = err {
            assert_eq!(e.kind::<SndEfwStatus>(), Some(SndEfwStatus::FlashBusy));
        } else {
            unreachable!();
        }

        // The read operation is always available.
        let mut data = [0;64];
        proto.flash_read(0, &mut data, TIMEOUT).unwrap();
    }

    #[test]
    fn flash_update_test() {
        let mut proto = TestProtocol::default();
        let count = proto.0.memory.len() / 4;
        (0..count).for_each(|i| {
            let pos = i * 4;
            proto.0.memory[pos..(pos + 4)].copy_from_slice(&(i as u32).to_be_bytes());
        });

        proto.flash_lock(false, TIMEOUT).unwrap();
        proto.flash_erase(256, TIMEOUT).unwrap();

        // Check near the boundary between first and second blocks.
        let mut data = [0;16];
        proto.flash_read(248, &mut data, TIMEOUT).unwrap();
        assert_eq!(&data, &[62, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        // Check near the boundary between second and third blocks.
        let mut data = [0;8];
        proto.flash_read(504, &mut data, TIMEOUT).unwrap();
        assert_eq!(&data, &[0, 0, 128, 129, 130, 131, 132, 133]);

        // Update the second block.
        let mut data = [0;BLOCK_QUADLET_COUNT];
        data.iter_mut().enumerate().for_each(|(i, q)| *q = u32::MAX - i as u32);
        proto.flash_write(256, &data, TIMEOUT).unwrap();

        // Check near the boundary between second and third block.
        let mut data = [0;6];
        proto.flash_read(504, &mut data, TIMEOUT).unwrap();
        assert_eq!(&data, &[4294967233, 4294967232, 128, 129, 130, 131]);
    }

    struct StateMachine {
        // Here, the state machine is defined to have four blocks in which the first block is
        // immutable.
        memory: [u8;4 * BLOCK_SIZE],
        // At initial state, the memory is locked against erase and write operation.
        locked: bool,
    }

    impl Default for StateMachine {
        fn default() -> Self {
            Self {
                memory: [0;4 * BLOCK_SIZE],
                locked: true,
            }
        }
    }

    impl StateMachine {
        fn erase_block(&mut self, args: Option<&[u32]>, params: Option<&mut [u32]>)
            -> Result<(), Error>
        {
            if params != None {
                Err(Error::new(SndEfwStatus::BadParameter, "Useless parameter is given"))?;
            }
            if let Some(quads) = args {
                if quads.len() < 1 {
                    Err(Error::new(SndEfwStatus::BadCommand, "Argument is shorter than expected"))?;
                }
                // Align to block.
                let pos = (quads[0] as usize) / BLOCK_SIZE * BLOCK_SIZE;
                if pos == 0 {
                    Err(Error::new(SndEfwStatus::BadCommand, "The first block is immutable"))?;
                }
                if pos > self.memory.len() {
                    Err(Error::new(SndEfwStatus::BadCommand, "The offset is out of range"))?;
                }
                if self.locked {
                    Err(Error::new(SndEfwStatus::FlashBusy, "The flash memory is locked"))?;
                }
                self.memory[pos..(pos + BLOCK_SIZE)].fill(0);
                Ok(())
            } else {
                Err(Error::new(SndEfwStatus::BadCommand, "Useless argument is given"))
            }
        }

        fn read_data(&self, args: Option<&[u32]>, params: Option<&mut [u32]>) -> Result<(), Error> {
            if let Some(quads) = args {
                if quads.len() < 2 {
                    Err(Error::new(SndEfwStatus::BadCommand, "Argument is shorter than expected"))?;
                }
                let offset = quads[0] as usize;
                let count = quads[1] as usize;
                if count >= BLOCK_SIZE {
                    let msg = "The count of data should be less than size of block";
                    Err(Error::new(SndEfwStatus::BadCommand, &msg))?;
                }
                if offset + 4 * count > self.memory.len() {
                    Err(Error::new(SndEfwStatus::BadCommand, "The offset plus count is out of range"))?;
                }

                if let Some(quads) = params {
                    if quads.len() < 2 + count {
                        Err(Error::new(SndEfwStatus::BadParameter, "Parameter is shorter than expected"))?;
                    }
                    quads[0] = offset as u32;
                    quads[1] = count as u32;

                    let mut quadlet = [0;4];
                    quads[2..].iter_mut().enumerate().for_each(|(i, d)| {
                        let pos = offset as usize + i * 4;
                        quadlet.copy_from_slice(&self.memory[pos..(pos + 4)]);
                        *d = u32::from_be_bytes(quadlet);
                    });
                    Ok(())
                } else {
                    Err(Error::new(SndEfwStatus::BadParameter, "Parameter is required"))?
                }
            } else {
                Err(Error::new(SndEfwStatus::BadCommand, "Argument is required"))
            }
        }

        fn write_data(&mut self, args: Option<&[u32]>, params: Option<&mut [u32]>) -> Result<(), Error> {
            if params != None {
                Err(Error::new(SndEfwStatus::BadParameter, "Useless parameter is given"))?;
            }

            if let Some(quads) = args {
                if quads.len() < 3 {
                    Err(Error::new(SndEfwStatus::BadCommand, "Argument is shorter than expected"))?;
                }

                let offset = quads[0] as usize;
                if offset < BLOCK_SIZE {
                    Err(Error::new(SndEfwStatus::BadCommand, "The first block is immutable"))?;
                }

                let count = quads[1] as usize;
                let data = &quads[2..];
                if data.len() < count {
                    Err(Error::new(SndEfwStatus::BadCommand, "Contradiction between count and data"))?;
                }
                if data.len() > BLOCK_QUADLET_COUNT {
                    let msg = "The count of data should be less than size of block";
                    Err(Error::new(SndEfwStatus::BadCommand, msg))?;
                }
                if offset + 4 * data.len() > self.memory.len() {
                    Err(Error::new(SndEfwStatus::BadCommand, "The offset plus length is out of range"))?;
                }
                if self.locked {
                    Err(Error::new(SndEfwStatus::FlashBusy, "The flash memory is locked"))?;
                }
                data.iter().enumerate().for_each(|(i, d)| {
                    let pos = offset + i * 4;
                    self.memory[pos..(pos + 4)].copy_from_slice(&d.to_be_bytes());
                });
                Ok(())
            } else {
                Err(Error::new(SndEfwStatus::BadCommand, "Argument is required"))
            }
        }

        fn get_status(&self, args: Option<&[u32]>, params: Option<&mut [u32]>)
            -> Result<(), Error>
        {
            if args != None {
                Err(Error::new(SndEfwStatus::BadCommand, "Useless argument is given"))?;
            }
            if params != None {
                Err(Error::new(SndEfwStatus::BadParameter, "Useless parameter is given"))?;
            }
            if self.locked {
                Err(Error::new(SndEfwStatus::FlashBusy, "The flash memory is locked"))?;
            }
            Ok(())
        }

        fn get_session_base(&self, args: Option<&[u32]>, params: Option<&mut [u32]>)
            -> Result<(), Error>
        {
            if args != None {
                Err(Error::new(SndEfwStatus::BadCommand, "Useless argument is given"))?;
            }
            if let Some(quads) = params {
                if quads.len() < 1 {
                    Err(Error::new(SndEfwStatus::BadParameter, "Parameter is shorter than expected"))?;
                }
                quads[0] = BLOCK_SIZE as u32;
                Ok(())
            } else {
                Err(Error::new(SndEfwStatus::BadParameter, "Parameter is required"))
            }
        }

        fn lock_memory(&mut self, args: Option<&[u32]>, params: Option<&mut [u32]>)
            -> Result<(), Error>
        {
            if params != None {
                Err(Error::new(SndEfwStatus::BadCommand, "Useless parameter is given"))?;
            }
            if let Some(quads) = args {
                if quads.len() < 1 {
                    Err(Error::new(SndEfwStatus::BadParameter, "Argument is shorter than expected"))?;
                }
                self.locked = quads[0] > 0;
                Ok(())
            } else {
                Err(Error::new(SndEfwStatus::BadCommand, "Argument is required"))
            }
        }
    }

    impl EfwProtocol for TestProtocol {
        fn transaction_sync(
            &mut self,
            category: u32,
            command: u32,
            args: Option<&[u32]>,
            params: Option<&mut [u32]>,
            _: u32,
        ) -> Result<(), glib::Error> {
            assert_eq!(category, CATEGORY_FLASH);
            match command {
                CMD_ERASE => self.0.erase_block(args, params),
                CMD_READ => self.0.read_data(args, params),
                CMD_WRITE => self.0.write_data(args, params),
                CMD_STATUS => self.0.get_status(args, params),
                CMD_SESSION_BASE => self.0.get_session_base(args, params),
                CMD_LOCK => self.0.lock_memory(args, params),
                _ => unreachable!(),
            }
        }
    }
}
