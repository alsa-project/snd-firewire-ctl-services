The library crate includes implementation of protocol for BridgeCo. Enhanced Break Out Box
(BeBoB) solution and application devices connected to IEEE 1394 bus.

## BridgeCo. Enhanced Break Out Box (BeBoB) solution

BridgeCo. AG was founded in 2000 and seems to be reorganized around in 2005. The subsidiary
for professional audio was sold in 2009 and formed ArchWave AG. BridgeCo itself was acquired
by SMSC in 2011. Archwave AG was acquired by Riedel Communications GmbH in 2018.

BridgeCo (and ArchWave) provided DM1000 (launched in 2002), DM1000E (launched in 2004), DM1100
(launched in 2005), and DM1500 (launched in 2005) ASICs with software development kit (SDK) for
audio and music units in IEEE 1394 bus, as total solution called as "BridgeCo Enhanced Break Out
Box (BeBoB)". The unit allows the other nodes in IEEE 1394 bus to configure itself by operations
defined by IEC 61883-1/6, some AV/C general commands as well as extensible vendor unique commands.

The solution was widely applied by hardware vendors for their products to record/playback audio
as well as receive/transmit MIDI messages.

## ALSA bebob driver

Linux sound subsystem, a.k.a ALSA, provides loadable kernel module (`snd-bebob`) as driver for
the units. The driver maintains isochronous packet streams for audio frames and MIDI messages
since the other ASIC functions such as digital signal processing can be operated by user space
application directly. The crate is supplemental implementation for runtime program to satisfy
the purpose.

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

 * Apogee Ensemble
 * Behringer Firepower FCA610
 * Digidesign Mbox 2 Pro
 * Ego Systems Quatafire 610
 * Focusrite Saffire
 * Focusrite Saffire LE
 * Focusrite Saffire Pro 10 i/o
 * Focusrite Saffire Pro 26 i/o
 * Icon Firexon
 * M-Audio FireWire Solo
 * M-Audio FireWire Audiophile
 * M-Audio FireWire 410
 * M-Audio FireWire 1814
 * M-Audio Ozonic
 * M-Audio ProFire LightBridge
 * M-Audio ProjectMix I/O
 * PreSonus Firebox
 * PreSonus Firepod/FP10
 * PreSonus Inspire 1394
 * Roland Edirol FA-66
 * Roland Edirol FA-101
 * Stanton ScratchAmp in Final Scratch version 2
 * TerraTec Aureon 7.1 FW
 * TerraTec Phase 24 FW
 * TerraTec Phase X24 FW
 * TerraTec Phase 88 FW
 * Yamaha Go 44
 * Yamaha Go 46

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

### bco-bootloader-info.rs

This program retrieves information from node of target device by protocol defined by BridgeCo,
then print the information.

Without any command line argument, it prints help message and exit.

```sh
$ cargo run --bin bco-bootloader-info
Usage:
  bco-bootloader-info CDEV

  where:
    CDEV:       The path to special file of firewire character device, typically '/dev/fw1'.
```

Please run with an argument for firewire character device:

```sh
$ cargo run --bin bco-bootloader-info /dev/fw1
...
```
