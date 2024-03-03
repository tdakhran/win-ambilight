# win-ambilight

<p float="left" aligh="middle">
  <img src="media/wl-ambilight.gif" width="350" />
  <img src="media/panda.jpg" width="350" /> 
</p>

An ambient light for Windows 11.

> For Linux(Wayland) use [tdakhran/wl-ambilight](https://github.com/tdakhran/wl-ambilight).
>
> For Linux(X11) use [josh26turner/Ambilight](https://github.com/josh26turner/Ambilight).

## Hardware

* arduino IDE compatible device
* [FastLED](https://fastled.io/) compatible LED strip and power supply for it
* USB cable

## Software

There are 2 parts:

* `arduino driver` - receives LED color information via USB from PC service and controls LEDs
* `PC service` - computes LED color information and sends it to `arduino driver`

## How does it work?

```mermaid
sequenceDiagram
participant LEDStrip
participant Arduino
PCService->>Windows: screen capture request
Windows->>PCService: screen data
Note over PCService: Compute LED colors
PCService->>Arduino: Send LED colors over USB
Arduino->>LEDStrip: update colors over DATA PIN
```

## Usage

### Wiring

Glue LED strip (I used WS2812B 60LED/m) to the back side of the monitor.
Cut and solder the parts together. I used ESP32 as an Arduino device and attached it as well to the monitor. The monitor has a built-in USB hub and ESP32 is connected to it. The power supply I use for LEDs has specs 5V 40W.

<p float="left" aligh="middle">
  <img src="media/monitor.jpg" width="700" />
</p>

### Build

Use Arduino IDE to build a driver from `arduino` folder.

Use `cargo build --target x86_64-pc-windows-gnu` to build PC service in WSL2.

### Launch

Launch `win-ambilight.exe run -p COM3 -m 1`.
Works on Windows 11.

### Issues

#### Known

None.

#### Unknown

A lot. PRs are welcome.
