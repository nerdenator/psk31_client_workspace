/** TX bridge — forwards tx-status events from Rust backend to the UI */

import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export interface TxStatus {
  status: 'transmitting' | 'complete' | 'aborted' | string;
  progress: number;
}

export interface TxStatusCallbacks {
  onTransmitting?: (progress: number) => void;
  onComplete?: () => void;
  onAborted?: () => void;
  onError?: (message: string) => void;
}

let unlisten: UnlistenFn | null = null;

/** Start listening for TX status events and dispatch to callbacks */
export async function listenTxStatus(callbacks: TxStatusCallbacks): Promise<void> {
  // Clean up any previous listener
  stopTxBridge();

  unlisten = await listen<TxStatus>('tx-status', (event) => {
    const { status, progress } = event.payload;

    if (status === 'transmitting') {
      callbacks.onTransmitting?.(progress);
    } else if (status === 'complete') {
      callbacks.onComplete?.();
    } else if (status === 'aborted') {
      callbacks.onAborted?.();
    } else if (status.startsWith('error')) {
      callbacks.onError?.(status);
    }
  });
}

/** Stop listening for TX status events */
export function stopTxBridge(): void {
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
}
