/** TX control buttons (Send/Abort) — wired to the real PSK-31 TX backend */

import { startTx, stopTx } from '../services/backend-api';
import { listenTxStatus } from '../services/tx-bridge';

export function setupTxButtons(): void {
  const sendBtn = document.querySelector('.tx-btn-send') as HTMLButtonElement;
  const abortBtn = document.querySelector('.tx-btn-abort') as HTMLButtonElement;
  const txIndicator = document.querySelector('.tx-indicator') as HTMLElement;
  const pttIndicator = document.querySelector('.ptt-indicator') as HTMLElement;
  const pttStatus = document.querySelector('.ptt-status') as HTMLElement;
  const txInput = document.getElementById('tx-input') as HTMLTextAreaElement;

  if (!sendBtn || !abortBtn) return;

  sendBtn.addEventListener('click', async () => {
    const text = txInput.value.trim();
    if (text === '') return;

    // Get the selected audio output device
    const outputDropdown = document.getElementById('audio-output') as HTMLSelectElement;
    const deviceId = outputDropdown?.value;

    if (!deviceId) {
      console.error('No audio output device selected');
      return;
    }

    // Switch UI to TX state
    setTxState(true);

    try {
      await startTx(text, deviceId);
    } catch (err) {
      console.error('TX start failed:', err);
      setTxState(false);
    }
  });

  abortBtn.addEventListener('click', async () => {
    try {
      await stopTx();
    } catch (err) {
      console.error('TX stop failed:', err);
    }
    // Reset UI immediately — the tx-status event is a backup
    setTxState(false);
  });

  // Listen for TX status events from the backend
  listenTxStatus({
    onTransmitting: (_progress) => {
      // Could update a progress bar here in future
    },
    onComplete: () => {
      setTxState(false);
    },
    onAborted: () => {
      setTxState(false);
    },
    onError: (msg) => {
      console.error('TX error:', msg);
      setTxState(false);
    },
  });

  function setTxState(transmitting: boolean): void {
    if (transmitting) {
      txIndicator?.classList.add('active');
      pttIndicator?.classList.remove('rx');
      pttIndicator?.classList.add('tx');
      if (pttIndicator) pttIndicator.textContent = 'TX';
      if (pttStatus) pttStatus.textContent = 'Transmitting';
      sendBtn.disabled = true;
      abortBtn.disabled = false;
      txInput.disabled = true;
    } else {
      txIndicator?.classList.remove('active');
      pttIndicator?.classList.remove('tx');
      pttIndicator?.classList.add('rx');
      if (pttIndicator) pttIndicator.textContent = 'RX';
      if (pttStatus) pttStatus.textContent = 'Receiving';
      sendBtn.disabled = false;
      abortBtn.disabled = true;
      txInput.disabled = false;
    }
  }
}
