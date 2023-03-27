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

/// The parameter to erase content of flash for a block.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct EfwFlashErase {
    /// The offset in flash memory. It should be aligned by quadlet.
    pub offset: u32,
}

impl<O, P> EfwWhollyUpdatableParamsOperation<P, EfwFlashErase> for O
where
    O: EfwHardwareSpecification,
    P: EfwProtocolExtManual,
{
    fn update_wholly(proto: &mut P, states: &EfwFlashErase, timeout_ms: u32) -> Result<(), Error> {
        assert_eq!(states.offset % 4, 0);

        let args = [states.offset];
        let mut params = Vec::new();
        proto.transaction(CATEGORY_FLASH, CMD_ERASE, &args, &mut params, timeout_ms)
    }
}

/// The parameter to erase content of flash for a block.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct EfwFlashRead {
    /// The offset in flash memory. It should be aligned by quadlet.
    pub offset: u32,
    /// The content. The length should be less than 64.
    pub data: Vec<u32>,
}

impl<O, P> EfwWhollyCachableParamsOperation<P, EfwFlashRead> for O
where
    O: EfwHardwareSpecification,
    P: EfwProtocolExtManual,
{
    fn cache_wholly(
        proto: &mut P,
        states: &mut EfwFlashRead,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(states.offset % 4, 0);
        assert!(states.data.len() <= BLOCK_QUADLET_COUNT);

        let count = states.data.len();
        let args = [states.offset, count as u32];
        let mut params = vec![0; 2 + BLOCK_QUADLET_COUNT];

        proto
            .transaction(CATEGORY_FLASH, CMD_READ, &args, &mut params, timeout_ms)
            .map(|_| states.data.copy_from_slice(&params[2..(2 + count)]))
    }
}

/// The parameter to write content of flash.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct EfwFlashWrite {
    /// The offset in flash memory. It should be aligned by quadlet.
    pub offset: u32,
    /// The content. The length should be less than 64.
    pub data: Vec<u32>,
}

impl<O, P> EfwWhollyUpdatableParamsOperation<P, EfwFlashWrite> for O
where
    O: EfwHardwareSpecification,
    P: EfwProtocolExtManual,
{
    fn update_wholly(proto: &mut P, states: &EfwFlashWrite, timeout_ms: u32) -> Result<(), Error> {
        assert_eq!(states.offset % 4, 0);
        assert!(states.data.len() <= BLOCK_QUADLET_COUNT);

        let mut args = vec![0; 2 + BLOCK_QUADLET_COUNT];
        args[0] = states.offset;
        args[1] = states.data.len() as u32;
        args[2..(2 + states.data.len())].copy_from_slice(&states.data);

        let mut params = Vec::new();

        proto.transaction(CATEGORY_FLASH, CMD_WRITE, &args, &mut params, timeout_ms)
    }
}

/// The parameter to check whether the flash memory is locked or not.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EfwFlashState {
    /// Is unlocked.
    Unlocked,
    /// Is locked.
    Locked,
}

impl Default for EfwFlashState {
    fn default() -> Self {
        Self::Unlocked
    }
}

impl<O, P> EfwWhollyCachableParamsOperation<P, EfwFlashState> for O
where
    O: EfwHardwareSpecification,
    P: EfwProtocolExtManual,
{
    fn cache_wholly(
        proto: &mut P,
        states: &mut EfwFlashState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let args = Vec::new();
        let mut params = Vec::new();
        proto
            .transaction(CATEGORY_FLASH, CMD_STATUS, &args, &mut params, timeout_ms)
            .map(|_| *states = EfwFlashState::Unlocked)
            .or_else(|e| {
                if e.kind::<EfwProtocolError>() == Some(EfwProtocolError::FlashBusy) {
                    *states = EfwFlashState::Locked;
                    Ok(())
                } else {
                    Err(e)
                }
            })
    }
}

impl<O, P> EfwWhollyUpdatableParamsOperation<P, EfwFlashState> for O
where
    O: EfwHardwareSpecification,
    P: EfwProtocolExtManual,
{
    fn update_wholly(proto: &mut P, states: &EfwFlashState, timeout_ms: u32) -> Result<(), Error> {
        let args = vec![states.eq(&EfwFlashState::Locked) as u32];
        let mut params = Vec::new();
        proto.transaction(CATEGORY_FLASH, CMD_LOCK, &args, &mut params, timeout_ms)
    }
}

/// The parameter for session base in flash memory.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct EfwFlashSessionBase(pub u32);

impl<O, P> EfwWhollyCachableParamsOperation<P, EfwFlashSessionBase> for O
where
    O: EfwHardwareSpecification,
    P: EfwProtocolExtManual,
{
    fn cache_wholly(
        proto: &mut P,
        states: &mut EfwFlashSessionBase,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let args = Vec::new();
        let mut params = vec![0];
        proto
            .transaction(
                CATEGORY_FLASH,
                CMD_SESSION_BASE,
                &args,
                &mut params,
                timeout_ms,
            )
            .map(|_| states.0 = params[0])
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use glib::{translate::FromGlib, SignalHandlerId};
    use std::cell::RefCell;

    const BLOCK_SIZE: usize = 4 * BLOCK_QUADLET_COUNT as usize;
    const TIMEOUT: u32 = 10;

    struct TestProtocol;

    impl EfwHardwareSpecification for TestProtocol {
        const SUPPORTED_SAMPLING_RATES: &'static [u32] = &[];
        const SUPPORTED_SAMPLING_CLOCKS: &'static [ClkSrc] = &[];
        const CAPABILITIES: &'static [HwCap] = &[];
        const RX_CHANNEL_COUNTS: [usize; 3] = [0; 3];
        const TX_CHANNEL_COUNTS: [usize; 3] = [0; 3];
        const MONITOR_SOURCE_COUNT: usize = 0;
        const MONITOR_DESTINATION_COUNT: usize = 0;
        const MIDI_INPUT_COUNT: usize = 0;
        const MIDI_OUTPUT_COUNT: usize = 0;
        const PHYS_INPUT_GROUPS: &'static [(PhysGroupType, usize)] = &[];
        const PHYS_OUTPUT_GROUPS: &'static [(PhysGroupType, usize)] = &[];
    }

    #[derive(Default)]
    struct TestInstance(RefCell<StateMachine>);

    #[test]
    fn flash_lock_test() {
        let mut proto = TestInstance::default();

        // The initial status should be locked.
        let mut state = EfwFlashState::default();
        TestProtocol::cache_wholly(&mut proto, &mut state, TIMEOUT).unwrap();

        // The erase operation should be failed due to locked status.
        let erase = EfwFlashErase { offset: 256 };
        let err = TestProtocol::update_wholly(&mut proto, &erase, TIMEOUT).unwrap_err();
        assert_eq!(
            err.kind::<EfwProtocolError>(),
            Some(EfwProtocolError::FlashBusy)
        );

        // The write operation should be failed as well due to locked status.
        let write = EfwFlashWrite {
            offset: 256,
            data: vec![0; 16],
        };
        let err = TestProtocol::update_wholly(&mut proto, &write, TIMEOUT).unwrap_err();
        assert_eq!(
            err.kind::<EfwProtocolError>(),
            Some(EfwProtocolError::FlashBusy)
        );

        // The read operation is always available.
        let mut read = EfwFlashRead {
            offset: 0,
            data: vec![0; 64],
        };
        TestProtocol::cache_wholly(&mut proto, &mut read, TIMEOUT).unwrap();

        // Unlock it.
        let state = EfwFlashState::Unlocked;
        TestProtocol::update_wholly(&mut proto, &state, TIMEOUT).unwrap();

        // The erase operation should be available now.
        TestProtocol::update_wholly(&mut proto, &erase, TIMEOUT).unwrap();

        // The write operation should be available now.
        TestProtocol::update_wholly(&mut proto, &write, TIMEOUT).unwrap();

        // The read operation is always available.
        TestProtocol::cache_wholly(&mut proto, &mut read, TIMEOUT).unwrap();

        // Lock it.
        let state = EfwFlashState::Locked;
        TestProtocol::update_wholly(&mut proto, &state, TIMEOUT).unwrap();

        let mut state = EfwFlashState::default();
        TestProtocol::cache_wholly(&mut proto, &mut state, TIMEOUT).unwrap();
        assert_eq!(state, EfwFlashState::Locked);

        // The erase operation should be failed again;
        let err = TestProtocol::update_wholly(&mut proto, &erase, TIMEOUT).unwrap_err();
        assert_eq!(
            err.kind::<EfwProtocolError>(),
            Some(EfwProtocolError::FlashBusy)
        );

        // The write operation should be failed as well;
        let err = TestProtocol::update_wholly(&mut proto, &write, TIMEOUT).unwrap_err();
        assert_eq!(
            err.kind::<EfwProtocolError>(),
            Some(EfwProtocolError::FlashBusy)
        );

        // The read operation is always available.
        TestProtocol::cache_wholly(&mut proto, &mut read, TIMEOUT).unwrap();
    }

    #[test]
    fn flash_update_test() {
        let mut proto = TestInstance::default();

        let count = proto.0.borrow().memory.len() / 4;
        (0..count).for_each(|i| {
            let pos = i * 4;
            proto.0.borrow_mut().memory[pos..(pos + 4)].copy_from_slice(&(i as u32).to_be_bytes());
        });

        let state = EfwFlashState::Unlocked;
        TestProtocol::update_wholly(&mut proto, &state, TIMEOUT).unwrap();

        let erase = EfwFlashErase { offset: 256 };
        TestProtocol::update_wholly(&mut proto, &erase, TIMEOUT).unwrap();

        // Check near the boundary between first and second blocks.
        let mut read = EfwFlashRead {
            offset: 248,
            data: vec![0; 16],
        };
        TestProtocol::cache_wholly(&mut proto, &mut read, TIMEOUT).unwrap();

        // Check near the boundary between second and third blocks.
        let mut read = EfwFlashRead {
            offset: 504,
            data: vec![0; 8],
        };
        TestProtocol::cache_wholly(&mut proto, &mut read, TIMEOUT).unwrap();
        assert_eq!(&read.data, &[0, 0, 128, 129, 130, 131, 132, 133]);

        // Update the second block.
        let data = (0..BLOCK_QUADLET_COUNT)
            .map(|i| u32::MAX - i as u32)
            .collect();
        let write = EfwFlashWrite { offset: 256, data };
        TestProtocol::update_wholly(&mut proto, &write, TIMEOUT).unwrap();

        // Check near the boundary between second and third block.
        let mut read = EfwFlashRead {
            offset: 504,
            data: vec![0; 6],
        };
        TestProtocol::cache_wholly(&mut proto, &mut read, TIMEOUT).unwrap();
        assert_eq!(&read.data, &[4294967233, 4294967232, 128, 129, 130, 131]);
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

    impl EfwProtocolExtManual for TestInstance {
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
