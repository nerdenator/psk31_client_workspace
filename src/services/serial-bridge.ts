/** Serial bridge — listens for backend-initiated serial disconnect events */

import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { resetSerialPanel } from '../components/serial-panel';
import { showToast } from '../components/toast';

interface SerialDisconnectedPayload {
  reason: string;
  port: string;
}

let unlisten: UnlistenFn | null = null;

export async function startSerialBridge(): Promise<void> {
  if (unlisten) return;

  unlisten = await listen<SerialDisconnectedPayload>('serial-disconnected', (event) => {
    const { port } = event.payload;
    const label = port ? `CAT disconnected: ${port}` : 'CAT disconnected';
    resetSerialPanel();
    showToast(label, 'error');
  });

  window.addEventListener('beforeunload', () => void unlisten?.());
}
