The crate includes Rust elements for the part of protocol defined by 1394 Trading Association
(1394 TA).

## Protocol documentation

The implemented protocol is documented in:

 * AV/C Digital Interface Command Set General Specification Version 4.2 (Sep. 1, 2004, 1394
   Trading Association, Document number 2004006)
 * Configuration ROM for AV/C Devices 1.0 (Dec. 12, 2000, 1394 Trading Association, TA Document
   1999027)

1394 TA was formed 1994 and decided to close its official operation 2015. As of 2021, it has been
dissolved and close its web site under URL `http://1394ta.org` in the end of year.

The document is itself available at [Internet Archive](https://archive.org/) service when seeking
URL `http://1394ta.org/specifications/` with enough care of cached date.

## Structure and enumerations

The crate consists of three parts; AV/C operations, some AV/C commands, and typical layout of
Configuration ROM defined in the documents.

### AV/C operations

AV/C operation uses "Function Control Protocol (FCP)" defined in IEC 61883-1 to send command frame
and receive response frame with two modes of transaction; immediate and deferred (see clause
"6 AV/C Operations"). The frame includes command type and response status, address, operation code,
and its operands (see clause "5 AV/C frames").

The address refers to unit and subunit described in clause "8 AV/C model".

The `Ta1394Avc` trait is good start to use the crate. `Ta1394Avc::transaction()` should be
implemented to perform FCP as well as handle the deferred transaction. The trait has default
implementation for methods to perform AV/C control, status, specific\_inquiry, and notify operation,
which uses the above implementation.

Unfortunately, actual devices include quirks against the design defined in specification. The
default implementation can be rewritten by implementator.

### AV/C commands

Each of AV/C command is expressed as converter between operands in frame and arbitrary
structure. The `AvcOp`, `AvcStatus`, `AvcControl`, and `AvcNotify` traits are used for the
conversion. The crate provides some implementations for documented AV/C commands below:

 * `UnitInfo` (clause "9.2 UNIT INFO command")
 * `SubunitInfo` (clause "9.3 SUBUNIT INFO command")
 * `VendorDependent` (clause "9.6 VENDOR-DEPENDENT commands")
 * `PlugInfo` (clause "10.1 PLUG INFO command")
 * `InputPlugSignalFormat` (clause "10.10 INPUT PLUG SIGNAL FORMAT command")
 * `OutputPlugSignalFormat` (clause "10.11 OUTPUT PLUG SIGNAL FORMAT command")

### Error handling

The generic `Ta1394AvcError` enumeration is used to express error of command composing,
communication failure, and response parsing. The implementator of `Ta1394Avc` trait should also
decide the type for error in communication failure.

### Typical layout of Configuration ROM

In typical layout defined by 1394 TA, root directory includes below entries in its order.

 * immediate entry for vendor ID
 * leaf entry for textual descriptor of vendor name
 * immediate entry for model ID
 * leaf entry for textual descriptor of model name
 * immediate entry for node capabilities
 * directory entry for unit

The unit directory includes below entries in its order:

 * immediate entry for specifier ID
 * immediate entry for version
 * immediate entry for model ID
 * leaf entry for textual descriptor of model name

The value of specifier ID entry is 0x00a02d, and the value of version entry is 0x010001.

To detect these fields, `Ta1394ConfigRom` trait is implemented for `ConfigRom` structure in
[ieee1212-config-rom](https://crates.io/crates/ieee1212-config-rom) crate. At present,
`VendorData` and `UnitData` is available as a result to detect the fields.

Besides, we can see the similar specification with slight differences in the other documents:

 * IEC 61883-1:1998
 * 1394-based Digital Camera Specification Version 1.04 (Aug. 9, 1996, 1394 Trading Association)
 * 1394-based Digital Camera Specification Version 1.20 (Jul. 23, 1998, 1394 Trading Association)
 * IIDC Digital Camera Control Specification Ver.1.30 (Jul. 25, 2000, 1394 Trading Association)
 * IIDC Digital Camera Control Specification Ver.1.31 (Feb. 2, 2004, 1394 Trading Association, TA
   Document 2003017)
 * IIDC Digital Camera Control Specification Ver.1.32 (Jul. 24, 2008, 1394 Trading Association,
   Document number 2007009)
 * IIDC2 Digital Camera Control Specification Ver.1.0.0 (Jan 26th, 2012, 1394 Trading Association,
   TS2011001)
 * IIDC2 Digital Camera Control Specification Ver.1.1.0 (May 19th, 2015, 1394 Trading Association,
   TS2015001)

Furthermore, some vendors designed specific layout for own purposes.

## License

The crate is released under [MIT license](https://spdx.org/licenses/MIT.html).

## Support

If finding issue, please file it in <https://github.com/alsa-project/snd-firewire-ctl-services/>.

## Contribution

If intending for code contribution, I would like the user and developer to take care of some
points before working.

It's well-known that the association promoted by several vendors tends to publishes
specifications and documentations with over-engineering for several reasons; e.g. rat race in
business or market. Beside the risk to include bug is exponentially increased when the code
base is larger and larger. It's not preferable for your work just to fulfill whole the
specifications and documentations.

The point is that there is actual requirement for the new code. For example, the crate includes
some AV/C commands to satisfy requirement of
[snd-firewire-ctl-services](https://github.com/alsa-project/snd-firewire-ctl-services/) project.
It's preferable for you to have actual application using the new code.
