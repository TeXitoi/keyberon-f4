# Keyberon port on the stm32f401 MCU [![Build status](https://travis-ci.org/TeXitoi/keyberon-f4.svg?branch=master)](https://travis-ci.org/TeXitoi/keyberon-f4)

## Hardware

https://www.aliexpress.com/item/4000069263843.html

## Flashing using DFU

```shell
cargo objcopy --bin keyberon-f4 --release -- -O binary keyberon.bin
sudo dfu-util -d 0483:df11 -a 0 --dfuse-address 0x08000000 -D keyberon.bin 
```

## Functionalities

User led is Caps Lock, user button (between PA0 and ground) is space.
