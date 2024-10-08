# Rust Minidisc
[![Lib.rs Version](https://img.shields.io/crates/v/minidisc?style=for-the-badge&logo=rust&label=lib.rs&color=%23a68bfc)](https://lib.rs/crates/minidisc)
[![docs.rs](https://img.shields.io/docsrs/minidisc?style=for-the-badge)](https://docs.rs/minidisc/)

A library for controlling and interfacing with [MiniDisc](https://en.wikipedia.org/wiki/MiniDisc) devices from within Rust programs. Compatible with many cross platform targets (including Web Assembly!) by using [cross-usb](https://github.com/G2-Games/cross-usb).

The feature set is very similar to that of [netmd-js](https://github.com/cybercase/netmd-js) which this library is inspired by. Development of this project was made much easier by the absolutely awesome [Web Minidisc project](https://github.com/asivery/webminidisc), [NetMD-exploits](https://github.com/asivery/netmd-exploits), and the C based [Linux Minidisc project](https://github.com/linux-minidisc/linux-minidisc). Go check those projects out!

> [!IMPORTANT]
> Documentation has not been finished and is a work in progress. Any contributions would be appreciated!

## Current Features
### NetMD
- [x] Track upload
- [x] Track management
- [x] Playback control
- [x] Group Management
- [x] Track download ([MZ-RH1](https://www.minidisc.wiki/equipment/sony/portable/mz-rh1) only)
- [ ] Factory Mode

### Hi-MD
- [ ] Track upload
- [ ] Track management
- [ ] Playback control
- [ ] Group Management

## Todo
- [ ] Exploits (from [NetMD-exploits](https://github.com/asivery/netmd-exploits))
- [ ] Hi-MD experimentation
- [ ] Documentation
- [ ] Better JS bindings
