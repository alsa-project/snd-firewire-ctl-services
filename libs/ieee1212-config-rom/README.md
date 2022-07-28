The crate includes structures, enumerations, traits and its implementations to process
content of Configuration ROM in IEEE 1212.

## Structures and enumerations

`ConfigRom` structure represents structured data of Configuration ROM. The structure
implements `std::convert::TryFrom<&[u8]>` to parse raw data of Configuration ROM. The
lifetime of `ConfigRom` structure is the same as the one of raw data, to save memory
consumption for string.

The `root` member of `ConfigRom` structure is a vector of `Entry` structure, which
represents directory entry. In the `Entry` structure, `key` member is typed as `KeyType`
for the type of key, and `data` member is typed as `EntryData` to dispatch four types
of data in the entry.

In IEEE 1212, text descriptor of leaf entry includes string information. `DescriptorLeaf`
structure is used to parse the descriptor.

For convenience, `EntryDataAccess` trait is available to access several type of data in
each entry by key.

## Usage

Add the following line to your Cargo.toml file:

```toml
[dependencies]
ieee1212-config-rom = "0.1"
```

[`ConfigRom`] structure is a good start to use the crate.

```rust
use ieee1212_config_rom::*;
use std::convert::TryFrom;

// Prepare raw data of Configuration ROM as array with u8 elements aligned to big endian.
let raw = [
    0x04, 0x04, 0x02, 0x91, 0x31, 0x33, 0x39, 0x34,
    0xf0, 0x00, 0xb2, 0x73, 0x08, 0x00, 0x28, 0x51,
    0x01, 0x00, 0x01, 0x4a, 0x00, 0x06, 0xa2, 0xd2,
    0x0c, 0x00, 0x83, 0xc0, 0x03, 0x00, 0x1f, 0x11,
    0x81, 0x00, 0x00, 0x04, 0x17, 0x02, 0x39, 0x01,
    0x81, 0x00, 0x00, 0x09, 0xd1, 0x00, 0x00, 0x0c,
    0x00, 0x06, 0x4c, 0xb7, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x4c, 0x69, 0x6e, 0x75,
    0x78, 0x20, 0x46, 0x69, 0x72, 0x65, 0x77, 0x69,
    0x72, 0x65, 0x00, 0x00, 0x00, 0x03, 0xff, 0x1c,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x4a, 0x75, 0x6a, 0x75, 0x00, 0x04, 0x66, 0xd5,
    0x12, 0x00, 0xa0, 0x2d, 0x13, 0x01, 0x00, 0x01,
    0x17, 0x02, 0x39, 0x03, 0x81, 0x00, 0x00, 0x01,
    0x00, 0x05, 0x40, 0x09, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x4c, 0x69, 0x6e, 0x75,
    0x78, 0x20, 0x41, 0x4c, 0x53, 0x41, 0x00, 0x00,
];

let config_rom = ConfigRom::try_from(&raw[..]).unwrap();

let expected = ConfigRom {
    bus_info: &raw[4..20],
    root: vec![
        Entry {
            key: KeyType::NodeCapabilities,
            data: EntryData::Immediate(0x0083c0),
        },
        Entry {
            key: KeyType::Vendor,
            data: EntryData::Immediate(0x001f11),
        },
        Entry {
            key: KeyType::Descriptor,
            data: EntryData::Leaf(&raw[52..76]),
        },
        Entry {
            key: KeyType::Model,
            data: EntryData::Immediate(0x023901),
        },
        Entry {
            key: KeyType::Descriptor,
            data: EntryData::Leaf(&raw[80..92]),
        },
        Entry {
            key: KeyType::Unit,
            data: EntryData::Directory(vec![
                Entry {
                    key: KeyType::SpecifierId,
                    data: EntryData::Immediate(0x00a02d),
                },
                Entry {
                    key: KeyType::Version,
                    data: EntryData::Immediate(0x010001),
                },
                Entry {
                    key: KeyType::Model,
                    data: EntryData::Immediate(0x023903),
                },
                Entry {
                    key: KeyType::Descriptor,
                    data: EntryData::Leaf(&raw[116..136]),
                },
            ]),
        },
    ],
};

assert_eq!(expected, config_rom);

// Without implementation of accessor trait.
let desc = DescriptorLeaf::try_from(&config_rom.root[2]).unwrap();
if let DescriptorData::Textual(d) = desc.data {
    assert_eq!("Linux Firewire", d.text);
} else {
    unreachable!();
}

// With implementation of accessor trait.
let model_id: u32 = config_rom.root[3].get(KeyType::Model).unwrap();
assert_eq!(0x023901, model_id);

let model_name: &str = config_rom.root[4].get(KeyType::Descriptor).unwrap();
assert_eq!("Juju", model_name);

let unit_entries: &[Entry] = config_rom.root[5].get(KeyType::Unit).unwrap();
let unit_model_name: &str = unit_entries[3].get(KeyType::Descriptor).unwrap();
assert_eq!("Linux ALSA", unit_model_name);
```

## Utilities

Some programs are available under `src/bin` directory.

### config-rom-parser.rs

This program parses raw data of Configuration ROM from stdin, or image file as arguments of
command line.

Without any command line argument, it prints help message and exit.

```sh
$ cargo run --bin config-rom-parser
Usage:
  config-rom-parser FILENAME | "-"

  where:
    FILENAME:       the name of file including the image of configuration ROM.
    "-":            the content of configuration ROM comes from STDIN.

  In both cases, the content of configuration ROM should be aligned to big endian.
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

## License

The ieee1212-config-rom crate is released under [MIT license](https://spdx.org/licenses/MIT.html).

## Support

If finding issue, please file it to <https://github.com/alsa-project/snd-firewire-ctl-services/>.
