# Unsplitted ergo Keyberon [![Build status](https://travis-ci.org/TeXitoi/keyberon-f4.svg?branch=master)](https://travis-ci.org/TeXitoi/keyberon-f4)

A handwired unsplitted ergo keyboard. It uses
[keyberon](https://github.com/TeXitoi/keyberon) for its firmware.

![Keyberon 56](images/keyberon-56.jpg)

## Bill of Materials

 - a [stm32f401 developement
   board](https://www.aliexpress.com/item/4000069263843.html)
 - a 3D printed [plate](cad/plate.stl)
 - a 3D printed [case](cad/case.stl)
 - 56 Cherry MX compatible keyboard switches
 - 56 Cherry MX compatible 1U keycaps
 - 56 1N4148 diodes
 - Polyurethane Enameled Copper Wire 0.2mm
 - 9 4mm M3 screws
 - 9 3mm M3 brass inserts
 - a USB-C cable
 - a soldering kit

Everything can be found for less than $60 new on Aliexpress.

## Compiling

Install the rust toolchain

```shell
curl https://sh.rustup.rs -sSf | sh
rustup target add thumbv7em-none-eabihf
rustup component add llvm-tools-preview
cargo install cargo-binutils
```

Compile the firmware

```shell
cargo objcopy --bin keyberon-f4 --release -- -O binary keyberon.bin
```

## Flashing using DFU

Put the developement board in DFU mode by pushing reset while pushing
boot, and then release boot. Then flash it:
```shell
dfu-util -d 0483:df11 -a 0 --dfuse-address 0x08000000 -D keyberon.bin
```

The LED on the board should react to caps lock (if you push caps lock
on another keyboard, the light should toggle), and the user button
send space.
