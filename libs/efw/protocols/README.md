The library crate includes implementation of protocol for Fireworks board module and application
devices connected to IEEE 1394 bus.

## Fireworks board module

Echo Digital Audio corporation equipped Fireworks board module in 2004. The main function of module
relies on two ICs; Texas Instruments TSB43CB43A (icelynx-Micro) and TMS320C67 DSP. The former IC
covers all of features required to IEEE 1394 and IEC 61883-1/6 with customized firmware. The latter
IC covers digital signal processing. Mackie Onyx 1200F and 400F, Echo Audio AudioFire 12 and 8 are
the first generation.

The second generation replaces the DSP with Xilinx Spartan XC35250E FPGA. Echo Audio AudioFire 12
and 8 (in 2009 or later), AudioFire 2, AudioFire 4, AudioFirePre8, and Robot Interface Pack (RIP)
of Gibson Robot Guitar series are at the latter generation.

The board module supports a pair of asynchronous transactions for command and response with
unique content of frame. The commands are categorized to several categories. The frame can
be delivers by plain asynchronous transaction to specific addresses as well as AV/C control
operation. The crate just implements the former transaction.

Echo Digital Audio corporation has US patent (US7599388B2) which describes usage of shared
memory with two banks between two processors to exchange data simultaneously between internal data
bus and serial data bus. It probably explains about the way to communicate between iceLynx-micro
and DSP/FPGA for audio frames.

## ALSA fireworks driver

Linux sound subsystem, a.k.a ALSA, provides loadable kernel module (`snd-fireworks`) as driver
for the units. The driver maintains isochronous packet streams for audio frames and MIDI
messages as well as the command response mechanism by a pair of asynchronous transactions since
the other internal functions such as digital signal processing can be operated by user space
application directly.

The driver allows the application to utilize the command and response mechanism by
executing system call.
[hitaki library](https://alsa-project.github.io/gobject-introspection-docs/hitaki/) has
`SndEfw` GObject class for the purpose. The command and response mechanism is required to
operate internal functions such as digital signal processing.

The crate depends on hitaki library so that runtime program can operate the internal functions
except for isochronous packet streams.

## Dependency

This is the list of dependent crates.

 * [glib crate](https://crates.io/crates/glib)
 * [hinawa crate](https://crates.io/crates/hinawa)
 * [hitaki crate](https://crates.io/crates/hitaki)

The glib and hinawa crates require some underlying system libraries

 * [glib library](https://docs.gtk.org/glib/)
 * [hinawa library](https://alsa-project.github.io/gobject-introspection-docs/hinawa/)
 * [hitaki library](https://alsa-project.github.io/gobject-introspection-docs/hitaki/)

The functions of Linux FireWire and Sound subsystem is called via hinawa and hitaki crates and
libraries to communicate with node in IEEE 1394 bus, thus the crate is not portable.

## Supported models

This is the list of models currently supported.

 * Mackie (Loud) Onyx 1200F
 * Mackie (Loud) Onyx 400F
 * Echo Audio Audiofire 12 (till Jul 2009)
 * Echo Audio Audiofire 8 (till Jul 2009)
 * Echo Audio Audiofire 12 (since Jul 2009)
 * Echo Audio Audiofire 8 (since Jul 2009)
 * Echo Audio Audiofire 2
 * Echo Audio Audiofire 4
 * Echo Audio Audiofire Pre8
 * Gibson Robot Interface Pack (RIP) for Robot Guitar series

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
driver provided by hardware vendor. Fortunately, developer got source code for command and response
mechanism endowed by Echo Digital Audio corporation, and permission to re-implement them to open
source software. Nevertheless it's natural not to work with your device since developer worked
with blackbox.
