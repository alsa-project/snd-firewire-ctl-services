// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use core::RuntimeOperation;

use std::str::FromStr;

pub fn parse_arg_as_u32(arg: &str) -> Result<u32, String> {
    u32::from_str(arg)
        .map_err(|e| format!("The first argument should be numeric number: {}, {}", e, arg))
}

pub trait ServiceCmd<'a, T, R> : Sized
    where R: RuntimeOperation<T>,
{
    const CMD_NAME: &'a str;
    const ARGS: &'a [(&'a str, &'a str)];
    fn parse_args(args: &[String]) -> Result<T, String>;

    fn print_help() {
        println!("
Usage:
  {}{}

  where",
                 Self::CMD_NAME,
                 &Self::ARGS.iter().fold(String::new(), |label, entry| label + " " + entry.0),
                 );

        Self::ARGS.iter().for_each(|entry| {
            println!("    {}: {}", entry.0, entry.1);
        })
    }

    fn run() {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let code =
            (if args.len() < Self::ARGS.len() {
                let msg = if Self::ARGS.len() == 1 {
                    format!("1 argument is required at least")
                } else {
                    format!("{} arguments are required at least", Self::ARGS.len())
                };
                Err(msg)
            } else {
                Self::parse_args(&args)
            })
            .and_then(|args| {
                R::new(args)
                    .map_err(|e| format!("Unsupported sound card: {}", e))
            })
            .and_then(|mut runtime| {
                runtime.listen()
                    .map_err(|e| format!("Fail to listen to events: {}", e))
                    .map(|_| runtime)
            })
            .and_then(|mut runtime| {
                runtime.run()
                    .map_err(|e| format!("Finish by error: {}", e))
            })
            .map(|_| libc::EXIT_SUCCESS)
            .unwrap_or_else(|msg| {
                eprintln!("{}", msg);
                Self::print_help();
                libc::EXIT_FAILURE
            });

        std::process::exit(code)
    }
}
