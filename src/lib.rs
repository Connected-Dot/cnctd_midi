use std::{thread::sleep, time::Duration};

use midir::{MidiInput, MidiOutput};
use anyhow::anyhow;
use tune::midi::ChannelMessage;

pub struct Midi;

impl Midi {

    pub fn get_devices() -> anyhow::Result<(Vec<String>, Vec<String>)> {
        let input = MidiInput::new("inputs")?;
        let output = MidiOutput::new("outputs")?;
        let mut input_devices: Vec<String> = vec![];
        let mut output_devices: Vec<String> = vec![];
        for (_i, p) in input.ports().iter().enumerate() {
            input_devices.push(input.port_name(p)?);
        }
        for (_i, p) in output.ports().iter().enumerate() {
            output_devices.push(output.port_name(p)?);
        }
        let inputs = input_devices.clone();
        let outputs = output_devices.clone();
        Ok((inputs, outputs))
    }

    pub async fn listen_to_device(device: &str) -> anyhow::Result<()> {
        let midi_input = MidiInput::new("inputs")?;
        let in_port = Self::find_port(&midi_input, device);
        match in_port {
            Some(in_port) => {
                let _conn_in = midi_input.connect(&in_port, "midir-read-input", 
                move |_stamp, message, _,| {
                    match ChannelMessage::from_raw_message(message) {
                        Some(msg) => println!("midi message: {:?}", msg),
                        None => println!("no message")
                    };
                }, 
                ());
                loop {
                    sleep(Duration::from_secs(10))
                }
            }
            None => {
                println!("no midi connected");
                Err(anyhow!("Midi device not connected"))
            }
        }
    }

    fn find_port<T>(midi_io: &T, device: &str) -> Option<T::Port> where T: midir::MidiIO {
        let mut device_port: Option<T::Port> = None;
        for port in midi_io.ports() {
            
            if let Ok(port_name) = midi_io.port_name(&port) {
                println!("port name: {} | device: {}", port_name, device);
                if port_name.contains(device.trim_matches('"')) {
                    device_port = Some(port);
                    break;
                }
            }
        }
        device_port
    }
    
}

