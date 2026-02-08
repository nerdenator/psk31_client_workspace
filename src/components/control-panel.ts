/** TX control buttons (Send/Abort) with mock TX state machine */

export function setupTxButtons(): void {
  const sendBtn = document.querySelector('.tx-btn-send') as HTMLButtonElement;
  const abortBtn = document.querySelector('.tx-btn-abort') as HTMLButtonElement;
  const txIndicator = document.querySelector('.tx-indicator') as HTMLElement;
  const pttIndicator = document.querySelector('.ptt-indicator') as HTMLElement;
  const pttStatus = document.querySelector('.ptt-status') as HTMLElement;
  const txInput = document.getElementById('tx-input') as HTMLTextAreaElement;

  if (sendBtn && abortBtn) {
    let txTimeout: number | null = null;

    sendBtn.addEventListener('click', () => {
      if (txInput.value.trim() === '') return;

      // Simulate TX state
      txIndicator.classList.add('active');
      pttIndicator.classList.remove('rx');
      pttIndicator.classList.add('tx');
      pttIndicator.textContent = 'TX';
      pttStatus.textContent = 'Transmitting';
      sendBtn.disabled = true;
      abortBtn.disabled = false;
      txInput.disabled = true;

      // Auto-return to RX after simulated transmission
      txTimeout = window.setTimeout(() => {
        txTimeout = null;
        resetToRx();
      }, 3000);
    });

    abortBtn.addEventListener('click', () => {
      if (txTimeout !== null) {
        clearTimeout(txTimeout);
        txTimeout = null;
      }
      resetToRx();
    });

    function resetToRx(): void {
      txIndicator.classList.remove('active');
      pttIndicator.classList.remove('tx');
      pttIndicator.classList.add('rx');
      pttIndicator.textContent = 'RX';
      pttStatus.textContent = 'Receiving';
      sendBtn.disabled = false;
      abortBtn.disabled = true;
      txInput.disabled = false;
    }
  }
}
