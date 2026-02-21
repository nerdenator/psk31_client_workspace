/** RX bridge — forwards decoded text events from Rust backend to the RX display */

import { listen, type UnlistenFn } from '@tauri-apps/api/event';

interface RxTextPayload {
  text: string;
}

let rxUnlisten: UnlistenFn | null = null;

/** Start listening for decoded RX text events */
export async function startRxBridge(onText: (text: string) => void): Promise<void> {
  await stopRxBridge();

  rxUnlisten = await listen<RxTextPayload>('rx-text', (event) => {
    onText(event.payload.text);
  });
}

/** Stop listening for RX text events */
export async function stopRxBridge(): Promise<void> {
  if (rxUnlisten) {
    rxUnlisten();
    rxUnlisten = null;
  }
}
