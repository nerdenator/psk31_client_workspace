/** RX (receive) display panel */

export function setupRxDisplay(): void {
  const clearBtn = document.querySelector('.rx-controls .rx-btn') as HTMLButtonElement;
  const rxContent = document.getElementById('rx-content') as HTMLElement;

  if (clearBtn && rxContent) {
    clearBtn.addEventListener('click', () => {
      rxContent.textContent = '';
    });
  }
}
