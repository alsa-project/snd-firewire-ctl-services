The crate includes Rust elements for the part of protocol defined by 1394 Trading Association
(1394 TA).

## Protocol documentation

The protocol is documented in:

 * AV/C Audio Subunit Specification 1.0 (Oct. 24, 2000, 1394 Trading Association, TA Document
   1999008)
 * Audio and Music Data Transmission Protocol 2.3 (Apr. 24, 2012, 1394 Trading Association,
   Document 2009013)

1394 TA was formed 1994 and decided to close its official operation 2015. As of 2021, it has been
dissolved and close its web site under URL `http://1394ta.org` in the end of year.

The document is itself available at [Internet Archive](https://archive.org/) service when seeking
URL `http://1394ta.org/specifications/` with enough care of cached date.

## Usage

Add the following line to your Cargo.toml file:

```toml
[dependencies]
ta1394-avc-audio = "0.2"
```

The crate consists of two parts; some AV/C commands and FDF format of A/M packet.

### AV/C commands

Some documented AV/C commands are available:

 * `AudioSelector` (clause "10.2 Selector function block")
 * `AudioFeature` (clause "10.3 Feature function block")
 * `AudioProcessing` (clause "10.4 Processing function block")

The commands should be given to implementation of `Ta1394Avc` trait provided in
[ta1394-avc-general](https://crates.io/crates/ta1394-avc-general) crate to perform AV/C operation.

### FDF format

The `AmdtpFdf` structure is provided to build and parse &[u8] for some events of Audio and Music
Data Transmission protocol.

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
