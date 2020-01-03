# OVERLAY

This overlay is needed because the go-rpio library freezes the machine hard.

Binary included. Be sure to add
`dtoverlay=generator`
to `/boot/config.txt`

To compile manually, run:

`dtc -I dts -O dtb -o generator.dtbo gpio-mostly-no-irq.dts`
