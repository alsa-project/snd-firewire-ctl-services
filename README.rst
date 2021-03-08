========================
snd-firewire-ctl-services
========================

2020/12/08
Takashi Sakamoto

Introduction
============

This project is a sub project in Advanced Linux Sound Architecture a.k.a ALSA,
to produce userspace service daemon for Audio and Music units on IEEE 1394 bus,
supported by drivers in ALSA firewire stack.

Executables
=================================

snd-firewire-digi00x-ctl-service
   For sound card bound to ALSA firewire-digi00x driver (snd-firewire-digi00x)
snd-firewire-tascam-ctl-service
   For sound card bound to ALSA firewire-tascam driver (snd-firewire-tascam), or TASCAM FE-8.
snd-fireworks-ctl-service
   For sound card bound to ALSA fireworks driver (snd-fireworks)
snd-firewire-motu-ctl-service
   For sound card bound to ALSA firewire-motu driver (snd-firewire-motu)
snd-oxfw-ctl-service
   For sound card bound to ALSA oxfw driver (snd-oxfw)
snd-bebob-ctl-service
   For sound card bound to ALSA bebob driver (snd-bebob)
snd-dice-ctl-service
   For sound card bound to ALSA dice driver (snd-dice)
snd-fireface-ctl-service
   For sound card bound to ALSA fireface driver (snd-fireface)

License
=======

GNU General Public License Version 3

Build dependencies
==================

* Rust programming language <https://www.rust-lang.org/>
* Cargo
* Some crates and their dependencies

  * glib crate in <https://gtk-rs.org/>
  * hinawa crate v0.3.0 in <https://github.com/alsa-project/hinawa-rs/>
  * alsactl/alsaseq crates v0.2.0 in <https://github.com/alsa-project/alsa-gobject-rs/>

Runtime dependencies
====================

* glib <https://developer.gnome.org/glib/>
* libhinawa v2.2.0 or later <https://github.com/alsa-project/libhinawa>
* alsa-gobject v0.2 or later <https://github.com/alsa-project/alsa-gobject/>

How to build
============

Build ::

    $ cargo build

Execute ::

    & cargo run --bin (the executable name) (the arguments of executable)

Supported devices
=================

Currently below devices are supported. If you would like to add support for
your device, please contact to developer.

* snd-firewire-digi00x-ctl-service

  * Digi 002
  * Digi 002 Rack
  * Digi 003
  * Digi 003 Rack
  * Digi 003 Rack+

* snd-firewire-tascam-ctl-service

  * Tascam FW-1884
  * Tascam FW-1082
  * Tascam FW-1804
  * Tascam FE-8

* snd-fireowrks-ctl-service

  * Mackie (Loud) Onyx 1200F
  * Mackie (Loud) Onyx 400F
  * Echo Audio Audiofire 12 (till Jul 2009)
  * Echo Audio Audiofire 8 (till Jul 2009)
  * Echo Audio Audiofire 12 (since Jul 2009)
  * Echo Audio Audiofire 8 (since Jul 2009)
  * Echo Audio Audiofire 2
  * Echo Audio Audiofire 4
  * Echo Audio Audiofire Pre8
  * Gibson Robot Interface Pack (RIP) for Robot Guitar series

* snd-firewire-motu-ctl-service

  * MOTU Traveler
  * MOTU 828mkII
  * MOTU UltraLite
  * MOTU 8pre
  * MOTU 828mk3
  * MOTU UltraLite mk3
  * MOTU 4pre
  * MOTU AudioExpress

* snd-oxfw-ctl-service

  * Tascam FireOne
  * Apogee Duet FireWire
  * Griffin FireWave
  * Lacie FireWire Speakers

* snd-bebob-ctl-service

  * Apogee Ensemble
  * M-Audio Ozonic
  * M-Audio FireWire Solo
  * M-Audio FireWire Audiophile
  * M-Audio FireWire 410
  * M-Audio ProFire LightBridge
  * M-Audio FireWire 1814
  * M-Audio ProjectMix I/O
  * Behringer FIREPOWER FCA610
  * Stanton ScratchAmp in Final Scratch version 2

* snd-dice-ctl-service

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
  * Focusrite Saffire Pro 26
  * PreSonus FireStudio
  * PreSonus FireStudio Project
  * PreSonus FireStudio Mobile
  * For the others, common controls are available. If supported, control extension is also available.

* snd-fireface-ctl-service

  * Fireface 800
  * Fireface 400

Supported protocols
===================

* IEEE 1212:2001 - IEEE Standard for a Control and Status Registers (CSR) Architecture for Microcomputer Buses https://ieeexplore.ieee.org/servlet/opac?punumber=8030
* Protocols defined by 1394 Trading Association http://1394ta.org/specifications/
   * Configuration ROM for AV/C Devices 1.0 (Dec. 2000, 1394 Trade Association)
   * AV/C Digital Interface Command Set General Specification Version 4.2 (September 1, 2004. TA Document 2004006)
   * Audio and Music Data Transmission Protocol 2.3 (April 24, 2012. Document 2009013)
   * AV/C Connection and Compatibility Management Specification 1.1 (March 19, 2003. TA Document 2002010)
   * AV/C Audio Subunit Specification 1.0 (October 24, 2000. TA Document 1999008)
   * AV/C Stream Format Information Specification 1.0 (May 24, 2002, TA Document 2001002)
   * AV/C Stream Format Information Specification 1.1 rev.5 (April 15, 2005. TA Document 2004008)
* Vendor specific protocols
   * Protocol for Digi 002/003 family of Digidesign
   * Protocol for FireWire series of TASCAM (TEAC)
   * Protocol for Fireworks board module of Echo Digital Audio
   * Protocol for Mark of the Unicorn (MOTU) FireWire series
   * Protocol for Oxford Semiconductor OXFW970/OXFW971 ASIC
   * Protocol for DM1000/DM1100/DM1500 ASIC in BridgeCo. Enhanced BreakOut Box (BeBoB)
   * Protocol for DiceII ASIC in Digital Interface Communication Engine (DICE)
   * Protocol extension for TCD2210/TCD2220 ASIC in Digital Interface Communication Engine (DICE)
   * Protocol for former models of Fireface series of RME GmbH
   * Protocol for latter models of Fireface series of RME GmbH

Design note
===========

Control model
-------------

.. image:: docs/control-model.png
   :alt: control model

Measure model
-------------

.. image:: docs/measure-model.png
   :alt: measure model

Notify model (with help of drivers in ALSA firewire stack)
-------------------------------------------------------------------

.. image:: docs/notify-model-a.png
   :alt: notify-a-model

Notify model (without any help of drivers in ALSA firewire stack)
-------------------------------------------------------------------

.. image:: docs/notify-model-b.png
   :alt: notify-b-model

Multi threading
---------------

.. image:: docs/overview.png
   :alt: overview
