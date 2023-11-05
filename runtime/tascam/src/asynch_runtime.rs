// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    super::{fe8_model::*, seq_cntr::*, *},
    runtime_core::dispatcher::*,
    nix::sys::signal::Signal,
    protocols::asynch::{fe8::*, *},
    std::marker::PhantomData,
    std::sync::mpsc,
};

pub type Fe8Runtime = AsynchRuntime<Fe8Model, Fe8Protocol>;

pub struct AsynchRuntime<S, T>
where
    S: SequencerCtlOperation<TascamExpander, T> + Default,
    T: MachineStateOperation,
{
    unit: (TascamExpander, FwNode),
    model: S,
    seq_cntr: SeqCntr,
    rx: mpsc::Receiver<AsyncUnitEvent>,
    tx: mpsc::SyncSender<AsyncUnitEvent>,
    dispatchers: Vec<Dispatcher>,
    converter: EventConverter<T>,
    _phantom: PhantomData<T>,
}

impl<S, T> Drop for AsynchRuntime<S, T>
where
    S: SequencerCtlOperation<TascamExpander, T> + Default,
    T: MachineStateOperation,
{
    fn drop(&mut self) {
        self.unit.0.unbind();

        let _ = self.model.fin(&mut self.unit.1);

        // At first, stop event loop in all of dispatchers to avoid queueing new events.
        for dispatcher in &mut self.dispatchers {
            dispatcher.stop();
        }

        // Next, consume all events in queue to release blocked thread for sender.
        for _ in self.rx.try_iter() {}

        // Finally Finish I/O threads.
        self.dispatchers.clear();
    }
}

enum AsyncUnitEvent {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Surface((u32, u32, u32)),
    SeqAppl(Vec<Event>),
}

const NODE_DISPATCHER_NAME: &str = "node event dispatcher";

impl<S, T> AsynchRuntime<S, T>
where
    S: SequencerCtlOperation<TascamExpander, T> + Default,
    T: MachineStateOperation,
{
    pub(crate) fn new(node: FwNode, name: String) -> Result<Self, Error> {
        let unit = TascamExpander::new();

        let seq_cntr = SeqCntr::new(&name)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        Ok(Self {
            unit: (unit, node),
            model: Default::default(),
            seq_cntr,
            tx,
            rx,
            dispatchers: Default::default(),
            converter: Default::default(),
            _phantom: Default::default(),
        })
    }

    pub(crate) fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;

        let enter = debug_span!("init").entered();
        self.unit.0.bind(&mut self.unit.1)?;
        self.seq_cntr.open_port()?;
        self.model.initialize_sequencer(&mut self.unit.1)?;
        enter.exit();

        Ok(())
    }

    pub(crate) fn run(&mut self) -> Result<(), Error> {
        let enter = debug_span!("listen").entered();
        self.unit.0.listen()?;
        enter.exit();

        let enter = debug_span!("event").entered();
        loop {
            let ev = match self.rx.recv() {
                Ok(ev) => ev,
                Err(_) => continue,
            };

            match ev {
                AsyncUnitEvent::Shutdown | AsyncUnitEvent::Disconnected => break,
                AsyncUnitEvent::BusReset(generation) => {
                    debug!("IEEE 1394 bus is updated: {}", generation);
                }
                AsyncUnitEvent::Surface((index, before, after)) => {
                    let _enter = debug_span!("surface").entered();
                    debug!(
                        "index: {}, before: 0x{:08x}, after: 0x{:08x}",
                        index, before, after
                    );
                    let _ = self.model.dispatch_surface_event(
                        &mut self.unit.0,
                        &mut self.unit.1,
                        &mut self.seq_cntr,
                        &self.converter,
                        index,
                        before,
                        after,
                    );
                }
                AsyncUnitEvent::SeqAppl(events) => {
                    let _enter = debug_span!("application").entered();
                    let _ = self.model.dispatch_appl_events(
                        &mut self.unit.1,
                        &mut self.seq_cntr,
                        &self.converter,
                        &events,
                    );
                }
            }
        }
        enter.exit();

        let enter = debug_span!("unlisten").entered();
        self.unit.0.unlisten();
        enter.exit();

        Ok(())
    }

    fn launch_node_event_dispatcher(&mut self) -> Result<(), Error> {
        // Use a dispatcher.
        let name = NODE_DISPATCHER_NAME.to_string();
        let mut dispatcher = Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_fw_node(&self.unit.1, move |_| {
            let _ = tx.send(AsyncUnitEvent::Disconnected);
        })?;

        let tx = self.tx.clone();
        self.unit.1.connect_bus_update(move |node| {
            let generation = node.generation();
            let _ = tx.send(AsyncUnitEvent::BusReset(generation));
        });

        let tx = self.tx.clone();
        dispatcher.attach_signal_handler(Signal::SIGINT, move || {
            let _ = tx.send(AsyncUnitEvent::Shutdown);
            source::Continue(false)
        });

        let tx = self.tx.clone();
        dispatcher.attach_snd_seq(&self.seq_cntr.client)?;
        self.seq_cntr
            .client
            .connect_handle_event(move |_, ev_cntr| {
                let events = ev_cntr.deserialize();
                let _ = tx.send(AsyncUnitEvent::SeqAppl(events));
            });

        let tx = self.tx.clone();
        self.unit.0.connect_changed(move |_, index, before, after| {
            let _ = tx.send(AsyncUnitEvent::Surface((index, before, after)));
        });

        self.dispatchers.push(dispatcher);

        Ok(())
    }
}
