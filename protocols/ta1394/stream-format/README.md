The crate includes Rust elements for the part of protocol defined by 1394 Trading Association
(1394 TA).

## Protocol documentation

The protocol is documented in:

 * AV/C Stream Format Information Specification 1.0 (May 24, 2002, 1394 Trading Association, TA
   Document 2001002)
 * AV/C Stream Format Information Specification 1.1 Revision 0.5 (April 15, 2005, 1394 Trading
   Association, TA Document 2004008) - unpublished

1394 TA was formed 1994 and decided to close its official operation 2015. As of 2021, it has been
dissolved and close its web site under URL `http://1394ta.org` in the end of year.

The published document is itself available at [Internet Archive](https://archive.org/) service when
URL `http://1394ta.org/specifications/` with enough care of cached date.

## Usage

Add the following line to your Cargo.toml file:

```toml
[dependencies]
ta1394-avc-stream-format = "0.2"
```

Some documented AV/C commands are available:

 * `ExtendedStreamFormatSingle` (clause "6.2.3 SINGLE subfunction" in specification v1.1)
 * `ExtendedStreamFormatList` (clause "6.2.4 LIST subfunction" in specification v1.1)

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
