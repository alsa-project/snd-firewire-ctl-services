The library crate includes implementation of protocol for models of MOTU FireWire series,
designed to audio and music unit in IEEE 1394 bus.

## MOTU FireWire series

The Mark of the Unicorn (MOTU) launched 828 in 2001 as its first model of FireWire series. The
model and 896 (launched in 2002) were the first generation of series in which the same hardware
design and protocol appear. The first model of second generation, 828 mkII, was launched in 2004.
The first model of third generation, 828 mk3 was launched in 2008.

In MOTU FireWire series, some models were launched for video processing purpose, while they are
not supported yet since protocol is unclear.

## Register DSP model and Command DSP model

All of models support notification by asynchronous transaction with quadlet message to
registered address. The message includes bit flags for some purposes; foot pedal event, and
so on. For further functions, the models in second and third generation also have two ways to
configure internal functions. In the crate, they are called as "register DSP" and "command DSP".

The register DSP models use asynchronous write transaction to address space of target node for
corresponding function. When user operates on-board function by hand, the change status is
notified in isochronous packet multiplexed with PCM frames and MIDI messages. The information
of hardware metering is also delivered by the same way.

The command DSP models use command to configure functions and notify status changes. The command
frame expresses current configuration. When user operates on-board function by hand, the command
frame is transmitted by asynchronous transaction to registered address. The other node transmits
asynchronous transaction with command frame to configure the functions. The information of
hardware metering is delivered by the same way as registered DSP model.

## ALSA firewire-motu driver

Linux sound subsystem, a.k.a ALSA, provides loadable kernel module (`snd-firewire-motu`) as
driver for the units. The driver maintains isochronous packet streams for audio frames and MIDI
messages as well as notification mechanism by asynchronous transaction since the other internal
functions such as digital signal processing can be operated by user space application directly.
For register DSP models and command DSP models, the driver caches state of device expressed in
the sequence of quadlet message in isochronous packet stream.

The driver allows the application to read the notified message and cache of state by executing
system call.
[hitaki library](https://alsa-project.github.io/gobject-introspection-docs/hitaki/) has
`SndMotu` GObject class for the purpose. Nevertheless, the crate has slight dependency on hitaki
crate about the `SndMotuRegisterDspParameter` structure. The operation to `SndMotu` class is
delegated to runtime program.

The crate is supplemental implementation for runtime program to operate the internal functions
except for isochronous packet streams.

## Dependency

This is the list of dependent crates.

 * [glib crate](https://crates.io/crates/glib)
 * [hinawa crate](https://crates.io/crates/hinawa)
 * [hitaki crate](https://crates.io/crates/hitaki)
 * [ieee1212-config-rom crate](https://crates.io/crates/ieee1212-config-rom)

The glib and hinawa crates require some underlying system libraries

 * [glib library](https://docs.gtk.org/glib/)
 * [hinawa library](https://alsa-project.github.io/gobject-introspection-docs/hinawa/)
 * [hitaki library](https://alsa-project.github.io/gobject-introspection-docs/hitaki/)

The functions of Linux FireWire and Sound subsystem is called via hinawa and hitaki crates and
libraries to communicate with node in IEEE 1394 bus, thus the crate is not portable.

## Supported models

This is the list of models currently supported.

 * MOTU 828
 * MOTU 896
 * MOTU Traveler
 * MOTU 828mkII
 * MOTU 896HD
 * MOTU UltraLite
 * MOTU 8pre
 * MOTU 4pre
 * MOTU AudioExpress
 * MOTU 828mk3 (FireWire only)
 * MOTU 828mk3 (Hybrid)
 * MOTU 896mk3 (FireWire only)
 * MOTU 896mk3 (Hybrid)
 * MOTU UltraLite mk3 (FireWire only)
 * MOTU UltraLite mk3 (Hybrid)
 * MOTU Traveler mk3
 * MOTU Track 16

Any help is welcome for:

 * MOTU 896 mk3 (FireWire only)
 * MOTU 896 mk3 (Hybrid)

## Status of the crate

The crate is developed and maintained by
[ALSA GObject Introspection team](https://alsa-project.github.io/gobject-introspection-docs/) for
[snd-firewire-ctl-services](https://github.com/alsa-project/snd-firewire-ctl-services/) project,
and not stable yet. The included Rust elements are likely changed without backward compatibility.

## License

The crate is released under
[GNU Lesser General Public License v3.0 or later](https://spdx.org/licenses/LGPL-3.0-or-later.html)
with respect to clause for reverse engineering.

## Support

If finding issue, please file it in <https://github.com/alsa-project/snd-firewire-ctl-services/>.

## Disclaimer

The implementation of protocol is developed by the way of reverse engineering; sniffing IEEE 1394
bus to which target device is connected, and analysis of the communication between the device and
driver provided by hardware vendor. It's natural not to work with your device since developer
worked with blackbox.
