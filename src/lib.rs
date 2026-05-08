use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use anyhow::anyhow;
use midi_message::MidiMessage;
use midir::{MidiInput, MidiOutput, MidiOutputConnection};

pub mod midi_message;

pub struct Midi;

/// A connected MIDI output port for sending messages (LED rings, button LEDs, etc.).
pub struct MidiOutputPort {
    conn: MidiOutputConnection,
}

impl MidiOutputPort {
    /// Send a raw MIDI message to the output device.
    pub fn send(&mut self, message: &[u8]) -> anyhow::Result<()> {
        self.conn
            .send(message)
            .map_err(|e| anyhow!("MIDI send error: {}", e))
    }

    /// Send a Control Change message (for LED rings on controllers like X-Touch Mini).
    pub fn send_cc(&mut self, channel: u8, controller: u8, value: u8) -> anyhow::Result<()> {
        let msg = MidiMessage::new(
            channel,
            midi_message::MidiMessageType::ControlChange { controller, value },
            None,
        );
        self.send(&msg.to_raw_message())
    }

    /// Send a Note On message (for button LEDs: velocity > 0 = on, 0 = off).
    pub fn send_note_on(&mut self, channel: u8, key: u8, velocity: u8) -> anyhow::Result<()> {
        let msg = MidiMessage::new(
            channel,
            midi_message::MidiMessageType::NoteOn { key, velocity },
            None,
        );
        self.send(&msg.to_raw_message())
    }
}

impl Midi {
    pub fn get_devices() -> anyhow::Result<(Vec<String>, Vec<String>)> {
        let input = MidiInput::new("inputs")?;
        let output = MidiOutput::new("outputs")?;
        let mut input_devices: Vec<String> = vec![];
        let mut output_devices: Vec<String> = vec![];
        for p in input.ports().iter() {
            if let Ok(name) = input.port_name(p) {
                input_devices.push(name);
            }
        }
        for p in output.ports().iter() {
            if let Ok(name) = output.port_name(p) {
                output_devices.push(name);
            }
        }
        Ok((input_devices, output_devices))
    }

    /// Connect to a MIDI output device by name (substring match).
    pub fn connect_output(device: &str) -> anyhow::Result<MidiOutputPort> {
        let midi_output = MidiOutput::new("cnctd-output")?;
        let port = Self::find_port(&midi_output, device)
            .ok_or_else(|| anyhow!("MIDI output device '{}' not found", device))?;
        let conn = midi_output
            .connect(&port, "cnctd-output")
            .map_err(|e| anyhow!("Failed to connect MIDI output: {}", e))?;
        Ok(MidiOutputPort { conn })
    }

    /// Listen for MIDI messages from a device, forwarding them to `tx`.
    /// Returns when the device disconnects (no longer found in device list)
    /// or when the sender's receiver is dropped.
    pub fn listen_to_device(device: &str, tx: mpsc::Sender<MidiMessage>) -> anyhow::Result<()> {
        let midi_input = MidiInput::new("inputs")?;
        let in_port = Self::find_port(&midi_input, device)
            .ok_or_else(|| anyhow!("MIDI device not connected"))?;

        let tx = Arc::new(Mutex::new(tx));

        let _conn_in = midi_input.connect(
            &in_port,
            "midir-read-input",
            move |stamp, message, _| {
                let tx = tx.clone();
                let message = message.to_owned();
                if let Some(msg) = MidiMessage::from_raw_message(&message, stamp) {
                    if let Ok(tx) = tx.lock() {
                        let _ = tx.send(msg);
                    }
                }
            },
            (),
        );

        // Periodically check if the device is still connected.
        // When it disappears (USB unplug), return so the caller can retry.
        let device_name = device.to_string();
        loop {
            thread::sleep(Duration::from_secs(3));
            match MidiInput::new("poll") {
                Ok(poll_input) => {
                    let still_present = poll_input.ports().iter().any(|p| {
                        poll_input
                            .port_name(p)
                            .map(|n| n.contains(&device_name))
                            .unwrap_or(false)
                    });
                    if !still_present {
                        return Err(anyhow!("MIDI device '{}' disconnected", device_name));
                    }
                }
                Err(_) => {
                    // Can't enumerate — assume disconnected
                    return Err(anyhow!("Failed to enumerate MIDI devices"));
                }
            }
        }
    }

    fn find_port<T>(midi_io: &T, device: &str) -> Option<T::Port>
    where
        T: midir::MidiIO,
    {
        for port in midi_io.ports() {
            if let Ok(port_name) = midi_io.port_name(&port) {
                if port_name.contains(device) {
                    return Some(port);
                }
            }
        }
        None
    }
}
