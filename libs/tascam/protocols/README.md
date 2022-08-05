The library crate includes implementation of protocols for models of TASCAM FireWire series,
designed to audio and music unit in IEEE 1394 bus.

## TASCAM FireWire series

As of 2022, TASCAM is an subsidiary of TEAC Corporation. FW-1884 was launched in 2003 as the first
TASCAM product for audio and music unit in IEEE 1394 bus. At the same time, FE-8 was launched as
expander unit for FW-1884. FW-1082 was launched in 2005 as console model smaller than FW-1884.
FW-1804 was launched in 2006 as rack model. The above models were provided by the joint project
between TASCAM and Frontier Design Group.

The models transmit state of hardware by subsequence of quadlet messages, while the way of
transmission is different between models. The most of models transmit the message in
isochronous packet multiplexed with PCM frame. FE-8 is an exception to transmits the message
by asynchronous transaction to registered address.

## ALSA firewire-tascam driver

Linux sound subsystem, a.k.a ALSA, provides loadable kernel module (`snd-firewire-tascam`) as
driver for the units except for FE-8. The driver maintains isochronous packet stream for
audio frames and the state messages as well as MIDI transmission by asynchronous transaction.

The driver allows user space application to retrieve the cached state and read notification for
the state change by executing system call.
[hitaki library](https://alsa-project.github.io/gobject-introspection-docs/hitaki/) has
`SndTascam` GObject class for the purpose. The class implements `TascamProtocol` GObject
interface, which is also implemented by `asynch::TascamExpander` structure so that FE-8 can be
supported as well.

Anyway, the crate is supplemental implementation for runtime program to configure digital
signal processing functions and console surfaces which are not implemented in the driver.

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

 * Tascam FW-1884
 * Tascam FW-1082
 * Tascam FW-1804
 * Tascam FE-8

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
