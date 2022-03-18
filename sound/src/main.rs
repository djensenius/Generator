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

fn run(command_receiver: crossbeam_channel::Receiver<Message>) -> Result<(), anyhow::Error> {
    let stream = stream_setup_for(sample_next, command_receiver.clone())?;
    stream.play()?;

    // Park the thread so out noise plays continuously until the app is closed
    std::thread::park();
    Ok(())
}

fn sample_next(o: &mut SampleRequestOptions, frequency: f32, amplitude: f32) -> f32 {
    o.tick();
    o.tone(frequency) * amplitude
}

pub struct SampleRequestOptions {
    pub sample_rate: f32,
    pub sample_clock: f32,
    pub nchannels: usize,
}

impl SampleRequestOptions {
    fn tone(&self, freq: f32) -> f32 {
        (self.sample_clock * freq * 2.0 * std::f32::consts::PI / self.sample_rate).sin()
    }
    fn tick(&mut self) {
        self.sample_clock = (self.sample_clock + 1.0) % self.sample_rate;
    }
}

pub fn stream_setup_for<F>(on_sample: F, command_receiver: crossbeam_channel::Receiver<Message>) -> Result<cpal::Stream, anyhow::Error>
where
    F: FnMut(&mut SampleRequestOptions, f32, f32) -> f32 + std::marker::Send + 'static + Copy,
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
    command_receiver: crossbeam_channel::Receiver<Message>,
) -> Result<cpal::Stream, anyhow::Error>
where
    T: cpal::Sample,
    F: FnMut(&mut SampleRequestOptions, f32, f32) -> f32 + std::marker::Send + 'static + Copy,
{
    let sample_rate = config.sample_rate.0 as f32;
    let sample_clock = 0f32;
    let nchannels = config.channels as usize;
    let mut request = SampleRequestOptions {
        sample_rate,
        sample_clock,
        nchannels,
    };
    let err_fn = |err| eprintln!("Error building output sound stream: {}", err);

    let mut frequency = 330.;
    let mut amplitude = 0.1;

    let stream = device.build_output_stream(
        config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            while let Ok(command) = command_receiver.try_recv() {
                println!("Received Message: {:?}", command);
                match command {
                    Message::Amplitude(val) => {
                        amplitude = val;
                    }

                    Message::Frequency(val) => {
                        frequency = val;
                    }
                }
            }
            on_window(output, &mut request, on_sample, frequency, amplitude)
        },
        err_fn,
    )?;

    Ok(stream)
}

fn on_window<T, F>(output: &mut [T], request: &mut SampleRequestOptions, mut on_sample: F, frequency: f32, amplitude: f32)
where
    T: cpal::Sample,
    F: FnMut(&mut SampleRequestOptions, f32, f32) -> f32 + std::marker::Send + 'static,
{
    for frame in output.chunks_mut(request.nchannels) {
        let value: T = cpal::Sample::from::<f32>(&on_sample(request, frequency, amplitude));
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

fn handle_packet(packet: OscPacket, command_sender: crossbeam_channel::Sender<Message>) {
    match packet {
        OscPacket::Message(msg) => {
            if msg.addr == String::from("/sine/amplitude") {
                for arg in msg.args {
                    match arg {
                        rosc::OscType::Float(x) => command_sender.send(Message::Amplitude(x)).unwrap(),
                        _ => (),
                    }
                }
            } else if msg.addr == String::from("/sine/frequency") {
                for arg in msg.args {
                    match arg {
                        rosc::OscType::Float(x) => command_sender.send(Message::Frequency(x)).unwrap(),
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
