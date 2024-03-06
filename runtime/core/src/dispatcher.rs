// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*,
    alsactl::{prelude::CardExt, Card},
    alsaseq::{prelude::UserClientExt, UserClient},
    glib::{prelude::IsA, source, ControlFlow, MainContext, MainLoop, Source},
    hinawa::{prelude::FwNodeExt, FwNode},
    hitaki::{prelude::AlsaFirewireExt, AlsaFirewire},
    nix::sys::signal,
    std::{sync::Arc, thread, time::Duration},
};

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
        let ctx = self.ev_loop.context();
        src.attach(Some(&ctx));
    }

    pub fn attach_signal_handler<F>(&mut self, signum: signal::Signal, cb: F)
    where
        F: FnMut() -> ControlFlow + Send + 'static,
    {
        let src =
            source::unix_signal_source_new(signum as i32, None, source::Priority::DEFAULT_IDLE, cb);

        self.attach_src_to_ctx(&src);
    }

    pub fn attach_snd_card<C, F>(&mut self, card: &C, disconnect_cb: F) -> Result<(), Error>
    where
        C: IsA<Card>,
        F: Fn(&C) + 'static,
    {
        let src = card.create_source()?;

        card.connect_handle_disconnection(disconnect_cb);

        self.attach_src_to_ctx(&src);

        Ok(())
    }

    pub fn attach_snd_seq<U>(&mut self, client: &U) -> Result<(), Error>
    where
        U: IsA<UserClient>,
    {
        let src = client.create_source()?;
        self.attach_src_to_ctx(&src);
        Ok(())
    }

    pub fn attach_alsa_firewire<U, F>(&mut self, unit: &U, disconnect_cb: F) -> Result<(), Error>
    where
        U: IsA<AlsaFirewire>,
        F: Fn(&U) + Send + 'static,
    {
        let src = unit.create_source()?;

        unit.connect_is_disconnected_notify(disconnect_cb);

        self.attach_src_to_ctx(&src);

        Ok(())
    }

    pub fn attach_fw_node<N, F>(&mut self, node: &N, disconnect_cb: F) -> Result<(), Error>
    where
        N: IsA<FwNode>,
        F: Fn(&N) + Send + Sync + 'static,
    {
        let src = node.create_source()?;

        node.connect_disconnected(disconnect_cb);

        self.attach_src_to_ctx(&src);

        Ok(())
    }

    pub fn attach_interval_handler<F>(&mut self, interval_msec: std::time::Duration, cb: F)
    where
        F: FnMut() -> ControlFlow + Send + 'static,
    {
        let src =
            source::timeout_source_new(interval_msec, None, source::Priority::DEFAULT_IDLE, cb);

        self.attach_src_to_ctx(&src);
    }
}
