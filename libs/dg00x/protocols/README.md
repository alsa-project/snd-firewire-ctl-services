The library crate includes implementation of protocol for Digidesign Digi 00x family, designed
to audio and music unit in IEEE 1394 bus.

## Digidesign Digi 00x family

As of 2022, Digidesign is one of brands managed by Avid Technology. Digi 002 was launched in
2002 as console model with audio interface. In 2003, Digi 002 Rack were launched as rack model.
Digi 003 and 003 Rack were launched in 2007.

The above models support mostly the same protocol to configure internal functions by
asynchronous transaction as well as audio data streaming by isochronous communication.

Digi 001 is out of target since it's not the system connected to IEEE 1394 bus.

## ALSA firewire-digi00x driver

Linux sound subsystem, a.k.a ALSA, provides loadable kernel module (`snd-firewire-digi00x`) as
driver for the units. The driver maintains isochronous packet streams for audio frames and MIDI
messages as well as notification mechanism by asynchronous transaction since the other internal
function such as digital signal processing can be operated by user space application directly.

The driver allows the application to read the notification by executing system call.
[hitaki library](https://alsa-project.github.io/gobject-introspection-docs/hitaki/) has
`SndDigi00x` GObject class for the purpose, while the crate is still independent of the library
to delegate the task to read the notification message into application program.

The crate is supplemental implementation for runtime program to operate the internal functions
except for isochronous packet streams.

## Dependency

This is the list of dependent crates.

 * [glib crate](https://crates.io/crates/glib)
 * [hinawa crate](https://crates.io/crates/hinawa)

The glib and hinawa crates require some underlying system libraries

 * [glib library](https://docs.gtk.org/glib/)
 * [hinawa library](https://alsa-project.github.io/gobject-introspection-docs/hinawa/)

The functions of Linux FireWire subsystem is called via hinawa crate and library to communicate
with node in IEEE 1394 bus, thus the crate is not portable.

## Supported models

This is the list of models currently supported.

 * Digi 002
 * Digi 002 Rack
 * Digi 003
 * Digi 003 Rack
 * Digi 003 Rack+

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
