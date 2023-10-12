The library crate includes implementation of protocols for Digital Interface Communication
Engine (DICE) solution and application devices connected to IEEE 1394 bus.

## Digital Interface Communication Engine (DICE)

TC Applied Technologies (TCAT) was established in 2003 as a result to split division of TC Group
for digital audio microelectronic and software technologies. The first product was DICE II ASIC
as single chip IEEE 1394 audio solution for pro and consumer applications. DICE II ASIC was
manufactured and distributed exclusively by WaveFront Semiconductor (formerly known as Alesis
Semiconductor). DICE Jr (TCD2200 ASIC) and DICE Mini (TCD2210 ASIC) seems to be released around
2005. DICE III (TCD3070 ASIC) was launched in 2014 with better jitter performance than the
previous.

DICE appeared to express solution including hardware abstraction layer in the ASICs and software
stack to operate by common application programming interface.

When Music Group acquired TC Group in 2015, the ownership of TCAT was also moved. One of group
company, CoolAudio International, distributes TCAT ASICs as well as WaveFront ICs. In 2017,
Music Group changed its name to Music Tribe.

## TCAT protocol

TCAT prepared options of software stack for two types of protocols; TCAT protocol, and
mLAN (Music Local Area Network)/OGT (Open Generic Transporter) protocols. DICE II ASIC was
utilized to Yamaha's mLAN 3rd generation; Yamaha n8/n12 (launched in 2007), Steinberg
MR816X/MR816CSX (launched in 2008).

The crate just supports TCAT protocol since mLAN/OGT protocols are closed. The protocol is
categorized to two parts; common and extension. All of DICE-based devices support the common
protocol, while the extension is just supported by DICE Jr. and DICE mini.

The remarkable point of the protocol is to manage memory space of node by several sections.
The content of section expresses configuration of the node. When the node voluntarily changed
the configuration or the other node changes the configuration by write transaction, the node
generates quadlet notification including bit flag corresponding to the section.

TC Electronic emulated the design of protocol for its Konnekt series, while some vendors
implements own protocol based on simple asynchronous transaction.

## ALSA dice driver

Linux sound subsystem, a.k.a ALSA, provides loadable kernel module (`snd-dice`) as driver for
the units. The driver maintains isochronous packet streams for audio frames and MIDI messages
as well as notification mechanism by asynchronous transaction since the other internal functions
of ASICs such as digital signal processing can be operated by user space application directly.

The driver allows the application to read the notification by executing system call.
[hitaki library](https://alsa-project.github.io/gobject-introspection-docs/hitaki/) has
`SndDice` GObject class for the purpose, while the crate is still independent of the library to
delegate the task to read the notification message into application program.

The crate is supplemental implementation for runtime program to operate the internal functions
except for isochronous packet streams.

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

 * M-Audio ProFire 2626
 * M-Audio ProFire 610
 * Avid Mbox 3 Pro
 * TC Electronic Konnekt 24d
 * TC Electronic Konnekt 8
 * TC Electronic Studio Konnekt 48
 * TC Electronic Konnekt Live
 * TC Electronic Desktop Konnekt 6
 * TC Electronic Impact Twin
 * TC Electronic Digital Konnekt x32
 * Alesis MultiMix 8/12/16 FireWire
 * Alesis iO 14
 * Alesis iO 26
 * Alesis MasterControl
 * Lexicon I-ONIX FW810s
 * Focusrite Saffire Pro 40
 * Focusrite Liquid Saffire 56
 * Focusrite Saffire Pro 24
 * Focusrite Saffire Pro 24 DSP
 * Focusrite Saffire Pro 14
 * Focusrite Saffire Pro 26
 * PreSonus FireStudio
 * PreSonus FireStudio Project
 * PreSonus FireStudio Tube
 * PreSonus FireStudio Mobile
 * Weiss Engineering ADC2
 * Weiss Engineering Vesta
 * Weiss Engineering DAC2, Minerva
 * Weiss Engineering AFI1
 * Weiss Engineering INT202, INT203, DAC1 FireWire option card
 * Weiss Engineering DAC202, Maya

For the other models, implementation for common and extension protocol is available without any
care of vendor's customization.

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

### tcat-general-parser.rs

This program retrieves information from node of target device according to general protocol,
then print the information.

Without any command line argument, it prints help message and exit.

```sh
$ cargo run --bin tcat-general-parser
Usage:
  tcat-general-parser CDEV

  where:
    CDEV:   The path to special file of firewire character device, typically '/dev/fw1'.
```

Please run with an argument for firewire character device:

```sh
$ cargo run --bin tcat-general-parser /dev/fw1
...
```

### tcat-extension-parser.rs

This program retrieves information from node of target device according to protocol extension,
then print the information.

Without any command line argument, it prints help message and exit.

```sh
$ cargo run --bin tcat-extension-parser
Usage:
  tcat-extension-parser CDEV

  where:
    CDEV:       The path to special file of firewire character device, typically '/dev/fw1'.
```

Please run with an argument for firewire character device:

```sh
$ cargo run --bin tcat-extension-parser /dev/fw1
...
```

### tcat-config-rom-parser.rs

This program parse the content of configuration ROM, then print information in it.

Without any command line argument, it prints help message and exit.

```sh
$ cargo run --bin tcat-config-rom-parser
Usage:
  tcat-config-rom-parser CDEV | "-"

  where:
    CDEV:       the path to special file of firewire character device, typically '/dev/fw1'.
    "-"         use STDIN for the content of configuration ROM to parse. It should be aligned to big endian.
```

Please run with an argument for firewire character device:

```sh
$ cargo run --bin tcat-config-rom-parser -- /dev/fw1
...
```

Or give content of configuration ROM via STDIN:

```sh
$ cat /sys/bus/firewire/devices/fw0/config_rom  | cargo run --bin tcat-config-rom-parser -- -
...
```

In the above case, the content should be aligned to big-endian order.
