extern crate anyhow;
extern crate clap;
extern crate cpal;
extern crate rosc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::thread;
use rosc::OscPacket;
use rosc::decoder;
use std::env;
use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;
use crossbeam_channel;

#[derive(Debug)]
pub enum Message {
    Frequency(f32),
    Amplitude(f32),
    Pan(f32),
    On(bool),
}

fn main() {
    // Start sound
    let (command_sender, command_receiver) = crossbeam_channel::bounded(1024);
    thread::spawn(move || {
        let _r = run(command_receiver.clone());
    });

    // OSC setup
    let args: Vec<String> = env::args().collect();
    let usage = format!("Usage {} IP:PORT", &args[0]);
    if args.len() < 2 {
        println!("{}", usage);
        ::std::process::exit(1)
    }
    let addr = match SocketAddrV4::from_str(&args[1]) {
        Ok(addr) => addr,
        Err(_) => panic!("{}", usage),
    };
    let sock = UdpSocket::bind(addr).unwrap();
    println!("Listening to {}", addr);

    let mut buf = [0u8; decoder::MTU];

    loop {
        match sock.recv_from(&mut buf) {
            Ok((size, _addr)) => {
                let packet = decoder::decode(&buf[..size]).unwrap();
                handle_packet(packet, command_sender.clone());
            }
            Err(e) => {
                println!("Error receiving from socket: {}", e);
                break;
            }
        }
    }
}

fn run(command_receiver: crossbeam_channel::Receiver<SoundMessage>) -> Result<(), anyhow::Error> {
    let stream = stream_setup_for(sample_next, command_receiver.clone())?;
    stream.play()?;

    // Park the thread so out noise plays continuously until the app is closed
    std::thread::park();
    Ok(())
}

fn sample_next(o: &mut SampleRequestOptions, instruments: [Instrument; 4]) -> f32 {
    o.tick();
    let mut f: f32 = 0.;
    for i in 0..instruments.len() {
        if instruments[i].on {
            if instruments[i].sound == Sound::Sine {
                f += o.sine(instruments[i].frequency) * instruments[i].amplitude;
            }
            if instruments[i].sound == Sound::Saw {
                f += o.saw(instruments[i].frequency) * instruments[i].amplitude;
            }
            if instruments[i].sound == Sound::Square {
                f += o.square(instruments[i].frequency) * instruments[i].amplitude;
            }
            if instruments[i].sound == Sound::Triangle {
                f += o.triangle(instruments[i].frequency) * instruments[i].amplitude;
            }
        }
    }
    f
}

pub struct SampleRequestOptions {
    pub sample_rate: f32,
    pub sample_clock: f32,
    pub nchannels: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Sound {
    Sine,
    Saw,
    Square,
    Triangle,
    None,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Instrument {
    sound: Sound,
    frequency: f32,
    amplitude: f32,
    pan: f32,
    on: bool
}

#[derive(Debug)]
pub struct SoundMessage {
    pub sound: Sound,
    pub message: Message,
}

impl SampleRequestOptions {
    fn sine(&self, freq: f32) -> f32 {
        // This is 1 because number of channels is 1?
        const TAU: f32 = 1_f32 * std::f32::consts::PI;
        let period: f32 = self.sample_clock / self.sample_rate;
        (freq * TAU * period).sin()
    }

    fn saw(&self, freq: f32) -> f32 {
        let period: f32 = self.sample_clock / self.sample_rate;
        (((freq * period) % 2_f32) - 0.5) * 1_f32
    }

    fn square(&self, freq: f32) -> f32 {
        const TAU: f32 = 1_f32 * std::f32::consts::PI;
        let period: f32 = self.sample_clock / self.sample_rate;
        match (freq * TAU * period).sin() {
            i if i > 0_f32 => 1_f32,
            _ => -1_f32,
        }
    }

    fn triangle(&self, freq: f32) -> f32 {
        const TAU: f32 = 1_f32 * std::f32::consts::PI;
        let period: f32 = self.sample_clock / self.sample_rate;
        (freq * TAU * period).sin().asin() * (2_f32 / std::f32::consts::PI)
    }

    fn tick(&mut self) {
        self.sample_clock = (self.sample_clock + 1.0) % self.sample_rate;
    }
}

pub fn stream_setup_for<F>(on_sample: F, command_receiver: crossbeam_channel::Receiver<SoundMessage>) -> Result<cpal::Stream, anyhow::Error>
where
    F: FnMut(&mut SampleRequestOptions, [Instrument; 4]) -> f32 + std::marker::Send + 'static + Copy,
{
    let (_host, device, config) = host_device_setup()?;

    match config.sample_format() {
        cpal::SampleFormat::F32 => stream_make::<f32, _>(&device, &config.into(), on_sample, command_receiver.clone()),
        cpal::SampleFormat::I16 => stream_make::<i16, _>(&device, &config.into(), on_sample, command_receiver.clone()),
        cpal::SampleFormat::U16 => stream_make::<u16, _>(&device, &config.into(), on_sample, command_receiver.clone()),
    }
}

pub fn host_device_setup(
) -> Result<(cpal::Host, cpal::Device, cpal::SupportedStreamConfig), anyhow::Error> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
    println!("Output device : {}", device.name()?);

    let config = device.default_output_config()?;
    println!("Default output config : {:?}", config);

    Ok((host, device, config))
}

pub fn stream_make<T, F>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    on_sample: F,
    command_receiver: crossbeam_channel::Receiver<SoundMessage>,
) -> Result<cpal::Stream, anyhow::Error>
where
    T: cpal::Sample,
    F: FnMut(&mut SampleRequestOptions, [Instrument; 4]) -> f32 + std::marker::Send + 'static + Copy,
{
    let sample_rate = config.sample_rate.0 as f32;
    let sample_clock = 0_f32;
    let nchannels = config.channels as usize;
    let mut request = SampleRequestOptions {
        sample_rate,
        sample_clock,
        nchannels,
    };
    let err_fn = |err| eprintln!("Error building output sound stream: {}", err);

    let mut sine  = Instrument {
        sound: Sound::Sine,
        frequency: 0.,
        amplitude: 0.,
        pan: 0.,
        on: false,
    };

    let mut saw  = Instrument {
        sound: Sound::Saw,
        frequency: 0.,
        amplitude: 0.,
        pan: 0.,
        on: false,
    };

    let mut square = Instrument {
        sound: Sound::Saw,
        frequency: 0.,
        amplitude: 0.,
        pan: 0.,
        on: false,
    };

    let mut triangle = Instrument {
        sound: Sound::Triangle,
        frequency: 0.,
        amplitude: 0.,
        pan: 0.,
        on: false,
    };

    let mut none = Instrument {
        sound: Sound::None,
        frequency: 0.,
        amplitude: 0.,
        pan: 0.,
        on: false,
    };

    let stream = device.build_output_stream(
        config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            while let Ok(command) = command_receiver.try_recv() {
                println!("Received Message: {:?}", command);
                let mut _modify_sound = &mut none;
                match command.sound {
                    Sound::Sine => {
                        _modify_sound = &mut sine;
                    }

                    Sound::Saw => {
                        _modify_sound = &mut saw;
                    }

                    Sound::Square => {
                        _modify_sound = &mut square;
                    }

                    Sound::Triangle => {
                        _modify_sound = &mut triangle;
                    }

                    Sound::None => {
                        _modify_sound = &mut none;
                    }
                }
                match command.message {
                    Message::Amplitude(val) => {
                        _modify_sound.amplitude = val;
                    }

                    Message::Frequency(val) => {
                        _modify_sound.frequency = val;
                    }

                    Message::Pan(val) => {
                        _modify_sound.pan = val;
                    }

                    Message::On(val) => {
                        _modify_sound.on = val;
                    }
                }
            }
            on_window(output, &mut request, on_sample, [sine.clone(), saw.clone(), square.clone(), triangle.clone()])
        },
        err_fn,
    )?;

    Ok(stream)
}

fn on_window<T, F>(output: &mut [T], request: &mut SampleRequestOptions, mut on_sample: F, instruments: [Instrument; 4])
where
    T: cpal::Sample,
    F: FnMut(&mut SampleRequestOptions, [Instrument; 4]) -> f32 + std::marker::Send + 'static,
{
    for frame in output.chunks_mut(request.nchannels) {
        // let value: T = cpal::Sample::from::<f32>(&on_sample(request, frequency, amplitude));
        let left: T = cpal::Sample::from::<f32>(&on_sample(request, instruments.clone()));
        let right: T = cpal::Sample::from::<f32>(&on_sample(request, instruments.clone()));
        for (channel, sample) in frame.iter_mut().enumerate() {
            // *sample = value;
            if channel & 1 == 0 {
                *sample = left;
            } else {
                *sample = right;
            }
        }
    }
}

fn handle_packet(packet: OscPacket, command_sender: crossbeam_channel::Sender<SoundMessage>) {
    match packet {
        OscPacket::Message(msg) => {
            let split = msg.addr.split("/");
            let vec = split.collect::<Vec<&str>>();
            let sound = vec[1];
            let variable = vec[2];

            let mut msg_sound: Sound = Sound::None;
            if sound == "sine" {
                msg_sound = Sound::Sine;
            } else if sound == "saw" {
                msg_sound = Sound::Saw;
            } else if sound == "square" {
                msg_sound = Sound::Square;
            } else if sound == "triangle" {
                msg_sound = Sound::Triangle;
            }


            println!("Sound {}, Variable {}", vec[1], vec[2]);
            if variable == String::from("amplitude") {
                for arg in msg.args {
                    match arg {
                        rosc::OscType::Float(x) => command_sender.send(SoundMessage {sound: msg_sound.clone(), message: Message::Amplitude(x)}).unwrap(),
                        _ => (),
                    }
                }
            } else if variable == String::from("frequency") {
                for arg in msg.args {
                    match arg {
                        rosc::OscType::Float(x) => command_sender.send(SoundMessage { sound: msg_sound.clone(), message: Message::Frequency(x)}).unwrap(),
                        _ => (),
                    }
                }
            } else if variable == "on" {
                for arg in msg.args {
                    match arg {
                        rosc::OscType::Bool(x) => command_sender.send(SoundMessage { sound: msg_sound.clone(), message: Message::On(x)}).unwrap(),
                        _ => (),
                    }
                }
            }
        }

        OscPacket::Bundle(bundle) => {
            println!("OSC Bundle: {:?}", bundle);
        }
    }
}
