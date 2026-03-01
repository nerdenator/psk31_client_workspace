/** TX (transmit) text input with character counter */

export function setupTxInput(): void {
  const txInput = document.getElementById('tx-input') as HTMLTextAreaElement;
  const charCount = document.querySelector('.tx-char-count .current') as HTMLElement;

  if (txInput && charCount) {
    txInput.addEventListener('input', () => {
      charCount.textContent = txInput.value.length.toString();
    });
  }
}
