/** Typed wrappers for all Tauri backend commands */

import { invoke } from '@tauri-apps/api/core';
import type { Configuration, AudioDeviceInfo, SerialPortInfo, RadioInfo } from '../types';

// Audio commands
export async function listAudioDevices(): Promise<AudioDeviceInfo[]> {
  return invoke('list_audio_devices');
}

// Serial commands
export async function listSerialPorts(): Promise<SerialPortInfo[]> {
  return invoke('list_serial_ports');
}

export async function connectSerial(port: string, baudRate: number): Promise<RadioInfo> {
  return invoke('connect_serial', { port, baud_rate: baudRate });
}

export async function disconnectSerial(): Promise<void> {
  return invoke('disconnect_serial');
}

// Radio commands
export async function pttOn(): Promise<void> {
  return invoke('ptt_on');
}

export async function pttOff(): Promise<void> {
  return invoke('ptt_off');
}

export async function getFrequency(): Promise<number> {
  return invoke('get_frequency');
}

export async function setFrequency(freqHz: number): Promise<void> {
  return invoke('set_frequency', { freq_hz: freqHz });
}

export async function getMode(): Promise<string> {
  return invoke('get_mode');
}

export async function setMode(mode: string): Promise<void> {
  return invoke('set_mode', { mode });
}

// Configuration commands
export async function saveConfiguration(config: Configuration): Promise<void> {
  return invoke('save_configuration', { config });
}

export async function loadConfiguration(name: string): Promise<Configuration> {
  return invoke('load_configuration', { name });
}

export async function listConfigurations(): Promise<string[]> {
  return invoke('list_configurations');
}

export async function deleteConfiguration(name: string): Promise<void> {
  return invoke('delete_configuration', { name });
}
