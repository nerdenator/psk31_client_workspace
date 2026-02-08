/** Shared type definitions matching Rust domain types */

export interface AudioDeviceInfo {
  id: string;
  name: string;
  is_input: boolean;
  is_default: boolean;
}

export interface SerialPortInfo {
  name: string;
  port_type: string;
}

export interface Configuration {
  name: string;
  audio_input: string | null;
  audio_output: string | null;
  serial_port: string | null;
  baud_rate: number;
  radio_type: string;
  carrier_freq: number;
}

export interface MenuEvent {
  id: string;
}
