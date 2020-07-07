// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use glib::Error;
use glib::{MainContext, MainLoop};

pub struct Dispatcher {
    name: String,
    th: Option<thread::JoinHandle<()>>,
    ev_loop: Arc<MainLoop>,
}

impl Drop for Dispatcher {
    fn drop(&mut self) {
        self.ev_loop.quit();

        if let Some(th) = self.th.take() {
            if th.join().is_err() {
                println!("Fail to join thread for {}.", self.name);
            }
        }
    }
}

impl Dispatcher {
    pub fn run(name: String) -> Result<Dispatcher, Error> {
        // Use own context.
        let ctx = MainContext::new();
        let ev_loop = Arc::new(MainLoop::new(Some(&ctx), false));

        // launch one thread to dispatch all events.
        let l = ev_loop.clone();
        let th = thread::spawn(move || {
            l.run();
            ()
        });

        // TODO: better mechanism to wait for the launch.
        loop {
            thread::sleep(Duration::from_millis(10));

            if ev_loop.is_running() {
                break;
            }
        }

        let th = Some(th);
        Ok(Dispatcher{name, th, ev_loop})
    }
}
