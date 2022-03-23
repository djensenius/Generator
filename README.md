# Generator

Version 2 of generator, in active development

## Sound

The `sound` directory contains code for generating sound.

To run, you need to pass in an ip address and port. For example if you want to use `cargo run` to run:

```
cargo run 127.0.0.1:8888
```

There are currently four types of sounds:
- Sine
- Saw
- Square
- Triangle

All sounds have four variables:
- frequency: 0-whatever (default 0),
- panning: -1 to 1 (default 0)
- amplitude: 0 to 1 (default 0)
- on: boolean (default false)

To activate sounds you need to send OSC commands, in the format:

`/instrument/variable value`

If you wish to use [sendosc](https://github.com/yoggy/sendosc) to send commands here are some examples:

```
sendosc 127.0.0.1 8888 /sine/frequency f 440
sendosc 127.0.0.1 8888 /square/amplitude f 0.2
sendosc 127.0.0.1 8888 /sine/on b 1
sendosc 127.0.0.1 8888 /sine/pan f 1.0
```

## Input

`input` contains code for reading input and sending to sound
