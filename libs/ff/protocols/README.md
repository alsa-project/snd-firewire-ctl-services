The library crate includes implementation of protocol for models of RME Fireface series connected
to IEEE 1394 bus.

## Fireface series

RME GmbH launched Fireface 800 in 2004 as its first product of audio and music unit in IEEE 1394
bus. It's categorized to the former model as well as Fireface 400 launched in 2007 to share
similar protocol which simply uses asynchronous write transaction to configure internal
functions.

The latter model includes Fireface UFX (launched in 2011), Fireface UCX (launched in 2012), and
Fireface 802 (launched in 2014). 4 byte command mechanism is adopted into the latter models to
configure internal DSP functions.

The latter models also supports Universal Serial Bus (USB), thus it's assumed that the 4 byte
command can be delivered by USB control packet when connecting to USB. Besides, the crate
supports IEEE 1394 bus as backend.

## ALSA fireface driver

Linux sound subsystem, a.k.a ALSA, provides loadable kernel module (`snd-fireface`) as driver for
the units. The driver maintains isochronous packet streams for audio frames and MIDI messages
since the other internal functions such as digital signal processing can be operated by user space
application directly.

The crate is supplemental implementation for runtime program to satisfy the purpose.

## Dependency

This is the list of dependent crates.

 * [glib crate](https://crates.io/crates/glib)
 * [hinawa crate](https://crates.io/crates/hinawa)
 * [ieee1212-config-rom crate](https://crates.io/crates/ieee1212-config-rom)

The glib and hinawa crates require some underlying system libraries

 * [glib library](https://docs.gtk.org/glib/)
 * [hinawa library](https://alsa-project.github.io/gobject-introspection-docs/hinawa/)

The functions of Linux FireWire subsystem is called via hinawa crate and library to communicate
with node in IEEE 1394 bus, thus the crate is not portable.

## Supported models

This is the list of models currently supported.

 * Fireface 800
 * Fireface 400
 * Fireface UCX
 * Fireface 802

Any help is welcome for:

 * Fireface UFX

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

## Utilities

Some programs are available under 'src/bin' directory.

### ff-config-rom-parser.rs

This program parses content of Configuration ROM specialized by RME, then print the information.

Without any command line argument, it prints help message and exit.

```sh
$ cargo run --bin ff-config-rom-parser
Usage:
  ff-config-rom-parser CDEV | "-"

  where:
    CDEV:       the path to special file of firewire character device, typically '/dev/fw1'.
    "-"         use STDIN for the content of configuration ROM to parse. It should be aligned to big endian.
```

Please run with an argument for firewire character device:

```sh
$ cargo run --bin ff-config-rom-parser /dev/fw1
...
```
