// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2022 Takashi Sakamoto

use {
    super::RuntimeOperation,
    alsactl::CardError,
    alsaseq::UserClientError,
    clap::Parser,
    glib::{Error, FileError},
    hinawa::FwNodeError,
    hitaki::AlsaFirewireError,
};

pub trait ServiceCmd<A, T, R>: Sized
where
    A: Parser,
    R: RuntimeOperation<T>,
{
    fn params(args: &A) -> T;

    fn run() {
        let code = A::try_parse()
            .map_err(|err| err.to_string())
            .map(|args| Self::params(&args))
            .and_then(|params| {
                R::new(params)
                    .and_then(|mut runtime| {
                        runtime.listen()?;
                        runtime.run()?;
                        Ok(libc::EXIT_SUCCESS)
                    })
                    .map_err(|err| specific_err_to_string(&err))
            })
            .unwrap_or_else(|msg| {
                eprintln!("{}", msg);
                libc::EXIT_FAILURE
            });

        std::process::exit(code)
    }
}

fn specific_err_to_string(e: &Error) -> String {
    let (domain, cause) = if let Some(error) = e.kind::<FileError>() {
        (
            "Linux file operation error",
            match error {
                FileError::Acces => "Access permission",
                FileError::Isdir => "Is directory",
                FileError::Noent => "Not exists",
                _ => "",
            },
        )
    } else if let Some(error) = e.kind::<AlsaFirewireError>() {
        (
            "ALSA HwDep operation error",
            match error {
                AlsaFirewireError::IsDisconnected => "Sound card is disconnected",
                AlsaFirewireError::IsUsed => "ALSA Hwdep device is already used",
                AlsaFirewireError::WrongClass => "Unit is not for the runtime",
                _ => "",
            },
        )
    } else if let Some(error) = e.kind::<FwNodeError>() {
        (
            "Linux FireWire node operation error",
            match error {
                FwNodeError::Disconnected => "Node is disconnected",
                _ => "",
            },
        )
    } else if let Some(error) = e.kind::<CardError>() {
        (
            "ALSA control operation error",
            match error {
                CardError::Disconnected => "Sound card is disconnected",
                _ => "",
            },
        )
    } else if e.is::<UserClientError>() {
        ("ALSA Sequencer operation error", "")
    } else {
        ("Unknown domain error", "")
    };
    format!("{}: {}, {}", domain, cause, e)
}
