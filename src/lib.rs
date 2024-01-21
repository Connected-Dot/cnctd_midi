use std::{sync::{mpsc, Arc, Mutex}, thread, time::Duration};

use midi_message::MidiMessage;
use midir::{MidiInput, MidiOutput};
use anyhow::anyhow;
// use tune::midi::ChannelMessage;

pub mod midi_message;
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

    pub fn listen_to_device(device: &str, tx: mpsc::Sender<MidiMessage>) -> anyhow::Result<()> {
        let midi_input = MidiInput::new("inputs")?;
        let in_port = Self::find_port(&midi_input, device).ok_or_else(|| anyhow!("Midi device not connected"))?;

        let tx = Arc::new(Mutex::new(tx)); // Wrap the sender in Arc and Mutex for safe sharing across threads

        let _conn_in = midi_input.connect(&in_port, "midir-read-input", 
            move |stamp, message, _,| {
                let tx = tx.clone(); // Clone the Arc to share across threads
                let message = message.to_owned(); // Clone the message data
                if let Some(msg) = MidiMessage::from_raw_message(&message, stamp) {
                    let tx = tx.lock().unwrap();
                    tx.send(msg).unwrap(); // Send the message through the channel
                }
            }, 
            (),
        );

        loop {
            thread::sleep(Duration::from_secs(10))
        }
    }

    fn find_port<T>(midi_io: &T, device: &str) -> Option<T::Port> where T: midir::MidiIO {
        let mut device_port: Option<T::Port> = None;
        for port in midi_io.ports() {
            
            if let Ok(port_name) = midi_io.port_name(&port) {
                // println!("port name: {} | device: {}", port_name, device);
                if port_name.contains(device) {
                    device_port = Some(port);
                    break;
                }
            }
        }
        device_port
    }
    
}

