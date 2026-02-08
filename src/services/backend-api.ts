/** Typed wrappers for all Tauri backend commands */

import { invoke } from '@tauri-apps/api/core';
import type { Configuration, AudioDeviceInfo, SerialPortInfo } from '../types';

// Audio commands
export async function listAudioDevices(): Promise<AudioDeviceInfo[]> {
  return invoke('list_audio_devices');
}

// Serial commands
export async function listSerialPorts(): Promise<SerialPortInfo[]> {
  return invoke('list_serial_ports');
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
