use std::time::{self, SystemTime, UNIX_EPOCH};

use serde::{Serialize, Deserialize};

const NOTE_OFF: u8 = 0b1000;
const NOTE_ON: u8 = 0b1001;
const POLYPHONIC_KEY_PRESSURE: u8 = 0b1010;
const CONTROL_CHANGE: u8 = 0b1011;
const PROGRAM_CHANGE: u8 = 0b1100;
const CHANNEL_PRESSURE: u8 = 0b1101;
const PITCH_BEND_CHANGE: u8 = 0b1110;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct MidiMessage {
    pub channel: u8,
    pub message_type: MidiMessageType,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum MidiMessageType {
    NoteOff { key: u8, velocity: u8 },
    NoteOn { key: u8, velocity: u8 },
    PolyphonicKeyPressure { key: u8, pressure: u8 },
    ControlChange { controller: u8, value: u8 },
    ProgramChange { program: u8 },
    ChannelPressure { pressure: u8 },
    PitchBendChange { value: i16 },
}

impl MidiMessage {
    pub fn new(channel: u8, message_type: MidiMessageType, stamp: Option<u64>) -> Self {
        let timestamp = if stamp.is_some() {
            stamp.unwrap()
        } else {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis() as u64
        };
        Self {
            channel,
            message_type,
            timestamp,
        }
    }

    pub fn from_raw_message(message: &[u8], timestamp: u64) -> Option<Self> {
        let status_byte = *message.first()?;
        let channel = status_byte & 0b0000_1111;
        let action = status_byte >> 4;
        let message_type = match action {
            NOTE_OFF => MidiMessageType::NoteOff {
                key: *message.get(1)?,
                velocity: *message.get(2)?,
            },
            NOTE_ON => MidiMessageType::NoteOn {
                key: *message.get(1)?,
                velocity: *message.get(2)?,
            },
            POLYPHONIC_KEY_PRESSURE => MidiMessageType::PolyphonicKeyPressure {
                key: *message.get(1)?,
                pressure: *message.get(2)?,
            },
            CONTROL_CHANGE => MidiMessageType::ControlChange {
                controller: *message.get(1)?,
                value: *message.get(2)?,
            },
            PROGRAM_CHANGE => MidiMessageType::ProgramChange {
                program: *message.get(1)?,
            },
            CHANNEL_PRESSURE => MidiMessageType::ChannelPressure {
                pressure: *message.get(1)?,
            },
            PITCH_BEND_CHANGE => MidiMessageType::PitchBendChange {
                value: i16::from_le_bytes([
                    *message.get(1)?,
                    *message.get(2)?,
                ]),
            },
            _ => return None,
        };
        Some(MidiMessage {
            channel,
            message_type,
            timestamp
        })
    }

    pub fn to_raw_message(&self) -> Vec<u8> {
        let status_byte = match self.message_type {
            MidiMessageType::NoteOff { .. } => NOTE_OFF,
            MidiMessageType::NoteOn { .. } => NOTE_ON,
            MidiMessageType::PolyphonicKeyPressure { .. } => POLYPHONIC_KEY_PRESSURE,
            MidiMessageType::ControlChange { .. } => CONTROL_CHANGE,
            MidiMessageType::ProgramChange { .. } => PROGRAM_CHANGE,
            MidiMessageType::ChannelPressure { .. } => CHANNEL_PRESSURE,
            MidiMessageType::PitchBendChange { .. } => PITCH_BEND_CHANGE,
        } << 4 | (self.channel & 0b0000_1111);

        let mut message = vec![status_byte];
        match self.message_type {
            MidiMessageType::NoteOff { key, velocity }
            | MidiMessageType::NoteOn { key, velocity } => {
                message.push(key);
                message.push(velocity);
            }
            MidiMessageType::PolyphonicKeyPressure { key, pressure }
            | MidiMessageType::ControlChange { controller: key, value: pressure } => {
                message.push(key);
                message.push(pressure);
            }
            MidiMessageType::ProgramChange { program }
            | MidiMessageType::ChannelPressure { pressure: program } => {
                message.push(program);
            }
            MidiMessageType::PitchBendChange { value } => {
                let value = value + 8192; // Center value at 8192 (0x2000)
                message.push((value & 0x7F) as u8); // LSB (7 bits)
                message.push(((value >> 7) & 0x7F) as u8); // MSB (7 bits)
            }
        }

        message
    }
}
