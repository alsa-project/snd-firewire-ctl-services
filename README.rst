========================
snd-firewire-ctl-services
========================

2020/07/07
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

License
=======

GNU General Public License Version 3

Build dependencies
==================

* Rust programming language <https://www.rust-lang.org/>
* Cargo
* Some crates and their dependencies

  * glib crate in <https://gtk-rs.org/>
  * hinawa crate v0.1.0 in <https://github.com/takaswie/hinawa-rs/>
  * alsactl/alsaseq crates v0.1.0 in <https://github.com/takaswie/alsa-gobject-rs/>

Runtime dependencies
====================

* glib <https://developer.gnome.org/glib/>
* libhinawa v2.0 or later <https://github.com/takaswie/libhinawa>
* alsa-gobject v0.1 or later <https://github.com/alsa-project/alsa-gobject/>

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

Supported protocols
===================

* IEEE 1212:2001 - IEEE Standard for a Control and Status Registers (CSR) Architecture for Microcomputer Buses https://ieeexplore.ieee.org/servlet/opac?punumber=8030
* Protocols defined by 1394 Trading Association http://1394ta.org/specifications/
   * Configuration ROM for AV/C Devices 1.0 (Dec. 2000, 1394 Trade Association)
* Vendor specific protocols
   * Protocol for Digi 002/003 family of Digidesign
   * Protocol for FireWire series of TASCAM (TEAC)

Design note
===========

Control model
-------------

.. image:: control-model.png
   :alt: control model

Monitor model
-------------

.. image:: monitor-model.png
   :alt: monitor model

Listener model (with help of drivers in ALSA firewire stack)
-------------------------------------------------------------------

.. image:: listener-model-a.png
   :alt: listener-a-model

Listener model (without any help of drivers in ALSA firewire stack)
-------------------------------------------------------------------

.. image:: listener-model-b.png
   :alt: listener-b-model

Multi threading
---------------

.. image:: overview.png
   :alt: overview
