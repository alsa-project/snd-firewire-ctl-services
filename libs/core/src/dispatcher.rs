// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use glib::Error;
use glib::IsA;
use glib::{source, MainContext, MainLoop, Source};

use nix::sys::signal;

use alsactl::CardExt;
use alsaseq::UserClientExt;
use hinawa::FwNodeExt;
use hinawa::SndUnitExt;

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
        Ok(Dispatcher { name, th, ev_loop })
    }

    pub fn stop(&mut self) {
        self.ev_loop.quit();
    }

    fn attach_src_to_ctx(&mut self, src: &Source) {
        let ctx = self.ev_loop.get_context();
        src.attach(Some(&ctx));
    }

    pub fn attach_signal_handler<F>(&mut self, signum: signal::Signal, cb: F)
    where
        F: FnMut() -> source::Continue + Send + 'static,
    {
        let src =
            source::unix_signal_source_new(signum as i32, None, source::PRIORITY_DEFAULT_IDLE, cb);

        self.attach_src_to_ctx(&src);
    }

    pub fn attach_snd_card<C, F>(&mut self, card: &C, disconnect_cb: F) -> Result<(), Error>
    where
        C: IsA<alsactl::Card>,
        F: Fn(&C) + 'static,
    {
        let src = card.create_source()?;

        card.connect_handle_disconnection(disconnect_cb);

        self.attach_src_to_ctx(&src);

        Ok(())
    }

    pub fn attach_snd_seq<U>(&mut self, client: &U) -> Result<(), Error>
    where
        U: IsA<alsaseq::UserClient>,
    {
        let src = client.create_source()?;
        self.attach_src_to_ctx(&src);
        Ok(())
    }

    pub fn attach_snd_unit<U, F>(&mut self, unit: &U, disconnect_cb: F) -> Result<(), Error>
    where
        U: IsA<hinawa::SndUnit>,
        F: Fn(&U) + Send + 'static,
    {
        let src = unit.create_source()?;

        unit.connect_disconnected(disconnect_cb);

        self.attach_src_to_ctx(&src);

        Ok(())
    }

    pub fn attach_fw_node<N, F>(&mut self, node: &N, disconnect_cb: F) -> Result<(), Error>
    where
        N: IsA<hinawa::FwNode>,
        F: Fn(&N) + Send + Sync + 'static,
    {
        let src = node.create_source()?;

        node.connect_disconnected(disconnect_cb);

        self.attach_src_to_ctx(&src);

        Ok(())
    }

    pub fn attach_interval_handler<F>(&mut self, interval_msec: std::time::Duration, cb: F)
    where
        F: FnMut() -> source::Continue + Send + 'static,
    {
        let msec = interval_msec.as_millis() as u32;
        let src = source::timeout_source_new(msec, None, source::PRIORITY_DEFAULT_IDLE, cb);

        self.attach_src_to_ctx(&src);
    }
}
