// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    super::{fe8_model::*, seq_cntr::*, *},
    core::dispatcher::*,
    nix::sys::signal::Signal,
    std::marker::PhantomData,
    std::sync::mpsc,
    tascam_protocols::asynch::{fe8::*, *},
};

pub type Fe8Runtime = AsynchRuntime<Fe8Model, Fe8Protocol, Fe8SurfaceState>;

pub struct AsynchRuntime<S, T, U>
where
    S: SequencerCtlOperation<T, U> + Default,
    T: MachineStateOperation + SurfaceImageOperation<U>,
{
    unit: (TascamExpander, FwNode),
    model: S,
    image: Vec<u32>,
    seq_cntr: SeqCntr,
    rx: mpsc::Receiver<AsyncUnitEvent>,
    tx: mpsc::SyncSender<AsyncUnitEvent>,
    dispatchers: Vec<Dispatcher>,
    _phantom0: PhantomData<T>,
    _phantom1: PhantomData<U>,
}

impl<S, T, U> Drop for AsynchRuntime<S, T, U>
where
    S: SequencerCtlOperation<T, U> + Default,
    T: MachineStateOperation + SurfaceImageOperation<U>,
{
    fn drop(&mut self) {
        self.unit.0.unbind();

        let _ = self.model.finalize_surface(&mut self.unit.1);

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
    SeqAppl(EventDataCtl),
}

const NODE_DISPATCHER_NAME: &str = "node event dispatcher";

impl<S, T, U> AsynchRuntime<S, T, U>
where
    S: SequencerCtlOperation<T, U> + Default,
    T: MachineStateOperation + SurfaceImageOperation<U>,
{
    pub fn new(node: FwNode, name: String) -> Result<Self, Error> {
        let unit = TascamExpander::new();

        let seq_cntr = SeqCntr::new(&name)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        Ok(Self {
            unit: (unit, node),
            model: Default::default(),
            image: vec![0u32; TascamExpander::QUADLET_COUNT],
            seq_cntr,
            tx,
            rx,
            dispatchers: Default::default(),
            _phantom0: Default::default(),
            _phantom1: Default::default(),
        })
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;

        self.unit.0.bind(&mut self.unit.1)?;

        self.seq_cntr.open_port()?;

        self.model.initialize_sequencer(&mut self.unit.1)?;

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Error> {
        self.unit.0.listen()?;

        loop {
            let ev = match self.rx.recv() {
                Ok(ev) => ev,
                Err(_) => continue,
            };

            match ev {
                AsyncUnitEvent::Shutdown | AsyncUnitEvent::Disconnected => break,
                AsyncUnitEvent::BusReset(generation) => {
                    println!("IEEE 1394 bus is updated: {}", generation);
                }
                AsyncUnitEvent::Surface((index, before, after)) => {
                    // Handle error of mutex lock as unrecoverable one.
                    self.unit.0.read_state(&mut self.image)?;
                    let _ = self.model.dispatch_surface_event(
                        &mut self.unit.1,
                        &mut self.seq_cntr,
                        &self.image,
                        index,
                        before,
                        after,
                    );
                }
                AsyncUnitEvent::SeqAppl(data) => {
                    let _ =
                        self.model
                            .dispatch_appl_event(&mut self.unit.1, &mut self.seq_cntr, &data);
                }
            }
        }

        self.unit.0.unlisten();

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
            let generation = node.get_property_generation();
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
                (0..ev_cntr.count_events())
                    .filter_map(|i| {
                        ev_cntr
                            .get_event_type(i)
                            .ok()
                            .filter(|ev_type| EventType::Controller.eq(ev_type))
                            .and_then(|_| ev_cntr.get_ctl_data(i).ok())
                    })
                    .for_each(|ctl_data| {
                        let data = AsyncUnitEvent::SeqAppl(ctl_data);
                        let _ = tx.send(data);
                    });
            });

        let tx = self.tx.clone();
        self.unit.0.connect_changed(move |_, index, before, after| {
            let _ = tx.send(AsyncUnitEvent::Surface((index, before, after)));
        });

        self.dispatchers.push(dispatcher);

        Ok(())
    }
}
