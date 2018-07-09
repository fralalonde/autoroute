autoroute
---------
USB MIDI command-line automatic patchbay for Linux. Connects USB MIDI devices together as they get plugged in, according to configuration.

No GUI means it can run on simple devices such as a Raspberry Pi. 

`autoroute (list | connect [--config=configfile] | install)`

## Usage

_Autoroute_ is simple to use:
- `list` shows all available USB MIDI device ports
- `connect` wires devices together according to the config file (`./autoroute.conf` is used by default)
- `install` sets up `autoroute` as a systemd service & udev callback that triggers on boot and everytime a USB MIDI device is connected or disconnected.

_Autoroute_ requires python 3.5. Built-in service installer requires `systemd`.

## Configuration

Config file entries can be of two types:
- `ignore` _name of device_ 
- `connect` _source device_ `->` _target device_

Lines starting with `#` are comments.
Device names have to match exactly the ones reported by the `autoroute list` command, excluding the `[device,port]`. 
If a configured device is not currently connected it is simply ignored.

```
# Ignored devices
ignore Timer
ignore Announce
ignore Midi Through Port-0

# Connections
connect USB Oxygen 49 MIDI 1 -> Arturia BeatStep MIDI 1
connect Arturia BeatStep MIDI 1 -> MicroBrute MIDI 1
connect Arturia BeatStep MIDI 1 -> GS-10 MIDI
connect Arturia BeatStep MIDI 1 -> GS-10 Control
connect Arturia BeatStep MIDI 1 -> R3 MIDI 1
```

## Thanks
Adapted from https://neuma.studio/rpi-as-midi-host.html to handle multi-port devices and fixed config.

## FAQ
Why choose that name?
- _Autoroute_ is french for _Highway_ and translates to german as _Autobahn_, which is a great [album](https://en.wikipedia.org/wiki/Autobahn_%28album%29).
