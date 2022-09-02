The library crate includes implementation of protocol for OXFW970 and OXFW971 ASICS and
application devices connected to IEEE 1394 bus.

## OXFW970 and OXFW971 ASICs

Oxford Semiconductor was established in 1992, and was acquired by PLX Technology in 2009. It
shipped two ASICs; OXFW970 (launched in 2004) and OXFW971 (launched in 2006), as low-cost
solution for multi channel isochronous streaming FireWire audio controller. They were applied
to some products from several vendors for audio and music unit in IEEE 1394 bus.

## ALSA oxfw driver

Linux sound subsystem, a.k.a ALSA, provides loadable kernel module (`snd-oxfw`) as driver for
the units. The driver maintains isochronous packet streams for audio frames and MIDI messages
since the other ASIC functions such as digital signal processing can be operated by user space
application directly.

The crate is supplemental implementation for runtime program to satisfy the purpose.

## Dependency

This is the list of dependent crates.

 * [glib crate](https://crates.io/crates/glib)
 * [hinawa crate](https://crates.io/crates/hinawa)
 * [ta1394-avc-general crate](https://crates.io/crates/ta1394-avc-general)
 * [ta1394-avc-audio crate](https://crates.io/crates/ta1394-avc-audio)
 * [ta1394-avc-stream-format crate](https://crates.io/crates/ta1394-avc-stream-format)
 * [ta1394-avc-ccm crate](https://crates.io/crates/ta1394-avc-ccm)

The glib and hinawa crates require some underlying system libraries

 * [glib library](https://docs.gtk.org/glib/)
 * [hinawa library](https://alsa-project.github.io/gobject-introspection-docs/hinawa/)

The functions of Linux FireWire subsystem is called via hinawa crate and library to communicate
with node in IEEE 1394 bus, thus the crate is not portable.

## Supported models

This is the list of models currently supported.

 * Tascam FireOne
 * Apogee Duet FireWire
 * Griffin FireWave
 * Lacie FireWire Speakers
 * Mackie Tapco Link.FireWire 4x6

The other models seem not to accept any operations to internal DSP functions from the outside.

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

## Utilities

Some programs are available under 'src/bin' directory.

### oxfw-info.rs

This program retrieves information from node of target device by protocol defined by Oxford
Semiconductor, then print the information.

Without any command line argument, it prints help message and exit.

```sh
Usage:
  oxfw-info CDEV

  where:
    CDEV:       The path to special file of firewire character device, typically '/dev/fw1'.
```

Please run with an argument for firewire character device:

```sh
$ cargo run --bin oxfw-info /dev/fw1
...
```
