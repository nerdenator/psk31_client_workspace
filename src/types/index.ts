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

export interface RadioInfo {
  port: string;
  baud_rate: number;
  frequency_hz: number;
  mode: string;
  connected: boolean;
}

export interface MenuEvent {
  id: string;
}

export interface ConnectionStatus {
  serial_connected: boolean;
  serial_port: string | null;
  audio_streaming: boolean;
  audio_device: string | null;
}
