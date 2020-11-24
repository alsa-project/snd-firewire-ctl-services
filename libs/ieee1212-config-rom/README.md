# ieee1212 - Parser for data of Configuration ROM in IEEE 1212

The crate includes structures, enumerations, traits and its implementations to process
content of Configuration ROM in IEEE 1212.

## Structures and enumerations

`ConfigRom` structure represents structured data of Configuration ROM. The structure
implements std::convert::TryFrom<&[u8]> to parse raw data of Configuration ROM. The
lifetime of `ConfigRom` structure is the same as the one of raw data, to save memory
consumption for string.

The `root` member of `ConfigRom` structure is a vector of `Entry` structure, which
represents directory entry. In the `Entry` structure, `key` member is typed as `KeyType`
for the type of key, and `data` member is typed as `EntryData` to dispatch four types
of data in the entry.

In IEEE 1212, text descriptor of leaf entry includes string information. `Descriptor`
structure is used to parse the descriptor.

For convenience, `EntryDataAccess` trait is available to access several type of data in
each entry by key.

## Usage

```rust
use ieee1212_config_rom::ConfigRom;
use ieee1212_config_rom::entry::{Entry, KeyType, EntryData, EntryDataAccess};
use ieee1212_config_rom::desc::{Descriptor, DescriptorData, TextualDescriptorData};
use std::convert::TryFrom;

// Prepare raw data of Configuration ROM as array with u8 elements aligned by big endian.
let raw =  [
    0x04, 0x04, 0x7f, 0x1a, 0x31, 0x33, 0x39, 0x34,
    0xf0, 0x00, 0xb2, 0x23, 0x08, 0x00, 0x28, 0x51,
    0x01, 0x00, 0x36, 0x22, 0x00, 0x05, 0x1b, 0x70,
    0x0c, 0x00, 0x83, 0xc0, 0x03, 0x00, 0x1f, 0x11,
    0x81, 0x00, 0x00, 0x03, 0x17, 0x02, 0x39, 0x01,
    0x81, 0x00, 0x00, 0x08, 0x00, 0x06, 0x4c, 0xb7,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x4c, 0x69, 0x6e, 0x75, 0x78, 0x20, 0x46, 0x69,
    0x72, 0x65, 0x77, 0x69, 0x72, 0x65, 0x00, 0x00,
    0x00, 0x03, 0xff, 0x1c, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x4a, 0x75, 0x6a, 0x75,
];

let config_rom = ConfigRom::try_from(&raw[..]).unwrap();
assert_eq!(KeyType::Vendor, config_rom.root[1].key);

if let EntryData::Immediate(value) = config_rom.root[1].data {
    assert_eq!(0x001f11, value);
} else {
    unreachable!();
}
let desc = Descriptor::try_from(&config_rom.root[2]).unwrap();
if let DescriptorData::Textual(d) = desc.data {
    assert_eq!("Linux Firewire", d.text);
} else {
    unreachable!();
}

let model_id = EntryDataAccess::<u32>::get(&config_rom.root[3], KeyType::Model).unwrap();
assert_eq!(0x023901, model_id);

let model_name = EntryDataAccess::<&str>::get(&config_rom.root[4], KeyType::Descriptor).unwrap();
assert_eq!("Juju", model_name);
```

## Utilities

Some programs are available under `src/bin` directory.

### src/bin/config-rom-parser

This program parses raw data of Configuration ROM from stdin, or image file as arguments of
command line.

Without any command line argument, it prints help message and exit.

```sh
$ cargo run --bin config-rom-parser
Usage:
  config-rom-parser FILENAME | "-"

  where:
    FILENAME:       the name of file for the image of configuration ROM to parse
    "-":            the content of configuration ROM to parse. It should be aligned to big endian.
```

For data of Configuration ROM in file:

```sh
$ cargo run --bin config-rom-parser -- /sys/bus/firewire/devices/fw0/config_rom
```

For data of Configuration ROM from stdin:

```sh
$ cat /sys/bus/firewire/devices/fw0/config_rom  | cargo run --bin config-rom-parser -- -
```

In both cases, the content to be parsed should be aligned to big-endian order.
