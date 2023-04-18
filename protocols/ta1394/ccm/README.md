The crate includes Rust elements for the part of protocol defined by 1394 Trading Association
(1394 TA).

## Protocol documentation

The protocol is documented in:

 * AV/C Connection and Compatibility Management Specification 1.1 (Mar. 19, 2003, 1394 Trading
   Association, TA Document 2002010)

1394 TA was formed 1994 and decided to close its official operation 2015. As of 2021, it has been
dissolved and close its web site under URL `http://1394ta.org` in the end of year.

The document is itself available at [Internet Archive](https://archive.org/) service when seeking
URL `http://1394ta.org/specifications/` with enough care of cached date.

## Usage

Add the following line to your Cargo.toml file:

```toml
[dependencies]
ta1394-avc-ccm = "0.2"
```

Some documented AV/C commands are available:

 * `SignalSource` (clause "7.1 SIGNAL SOURCE command")

The commands should be given to implementation of `Ta1394Avc` trait provided in
[ta1394-avc-general](https://crates.io/crates/ta1394-avc-general) crate to perform AV/C operation.

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
