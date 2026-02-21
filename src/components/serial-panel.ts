/** Serial panel — populates port dropdown, wires connect/disconnect button */

import { listSerialPorts, connectSerial, disconnectSerial } from '../services/backend-api';
import { setSerialState } from '../services/app-state';

/** Default baud rate for FT-991A */
const DEFAULT_BAUD_RATE = 38400;

/** How long the connect button stays green after a successful connection (ms) */
const SUCCESS_FLASH_MS = 10_000;

export function setupSerialPanel(): void {
  const dropdown = document.getElementById('serial-port') as HTMLSelectElement;
  const connectBtn = document.getElementById('serial-connect-btn') as HTMLButtonElement;
  const freqValue = document.querySelector('.sidebar .frequency-value') as HTMLElement;
  const freqMode = document.querySelector('.frequency-mode') as HTMLElement;
  const catDot = document.querySelector('#cat-status .status-dot') as HTMLElement;
  const catText = document.querySelector('#cat-status .status-text') as HTMLElement;

  if (!dropdown || !connectBtn) return;

  let connected = false;
  let flashTimeout: number | null = null;
  let connectionId = 0; // Incremented on each connect/disconnect to invalidate stale callbacks

  // Populate dropdown from backend on load
  populateDropdown(dropdown);

  connectBtn.addEventListener('click', async () => {
    if (connected) {
      // Disconnect
      try {
        await disconnectSerial();
      } catch (err) {
        console.error('Disconnect failed:', err);
      }
      resetUi();
      return;
    }

    // Connect
    const port = dropdown.value;
    if (!port) return;

    connectionId++;
    const thisConnection = connectionId;

    connectBtn.disabled = true;
    connectBtn.textContent = 'Connecting...';

    try {
      const info = await connectSerial(port, DEFAULT_BAUD_RATE);

      // If user disconnected (or reconnected) while we were awaiting, bail out
      if (thisConnection !== connectionId) return;

      connected = true;
      setSerialState(true, port);

      // Update frequency display (e.g. 14070000 → "14.070.000")
      if (freqValue) {
        freqValue.textContent = formatMhz(info.frequency_hz);
      }
      if (freqMode) {
        freqMode.textContent = info.mode;
      }

      // Update CAT status indicator
      if (catDot) {
        catDot.classList.remove('disconnected');
        catDot.classList.add('connected');
      }
      if (catText) {
        catText.classList.remove('disconnected');
        catText.classList.add('connected');
        catText.textContent = 'OK';
      }

      // Flash button green for 10s
      connectBtn.disabled = false;
      connectBtn.textContent = 'Connected';
      connectBtn.classList.add('connected');
      dropdown.disabled = true;

      if (flashTimeout) clearTimeout(flashTimeout);
      flashTimeout = window.setTimeout(() => {
        // Only update if this connection is still current
        if (thisConnection === connectionId) {
          connectBtn.classList.remove('connected');
          connectBtn.textContent = 'Disconnect';
        }
        flashTimeout = null;
      }, SUCCESS_FLASH_MS);
    } catch (err) {
      // If user disconnected while we were awaiting, bail out
      if (thisConnection !== connectionId) return;

      console.error('Connect failed:', err);
      connectBtn.disabled = false;
      connectBtn.textContent = 'Connect';

      // Show error in CAT status
      if (catDot) {
        catDot.classList.remove('connected');
        catDot.classList.add('disconnected');
      }
      if (catText) {
        catText.classList.remove('connected');
        catText.classList.add('disconnected');
        catText.textContent = `Error`;
      }
    }
  });

  function resetUi(): void {
    connected = false;
    setSerialState(false, null);
    connectionId++; // Invalidate any pending async callbacks or timeouts
    if (flashTimeout) {
      clearTimeout(flashTimeout);
      flashTimeout = null;
    }
    connectBtn.disabled = false;
    connectBtn.textContent = 'Connect';
    connectBtn.classList.remove('connected');
    dropdown.disabled = false;

    if (catDot) {
      catDot.classList.remove('connected');
      catDot.classList.add('disconnected');
    }
    if (catText) {
      catText.classList.remove('connected');
      catText.classList.add('disconnected');
      catText.textContent = 'N/C';
    }
  }
}

/** Fetch serial ports from backend and populate the dropdown */
async function populateDropdown(dropdown: HTMLSelectElement): Promise<void> {
  try {
    const ports = await listSerialPorts();

    // Clear existing options except the placeholder
    while (dropdown.options.length > 1) {
      dropdown.remove(1);
    }

    for (const port of ports) {
      const option = document.createElement('option');
      option.value = port.name;
      option.textContent = `${port.name} (${port.port_type})`;
      dropdown.appendChild(option);
    }
  } catch (err) {
    console.error('Failed to list serial ports:', err);
  }
}

/** Format Hz as MHz display string: 14070000 → "14.070.000" */
function formatMhz(hz: number): string {
  const mhzStr = Math.round(hz).toString().padStart(9, '0');
  // Split into groups: XX.XXX.XXX
  const a = mhzStr.slice(0, -6).replace(/^0+/, '') || '0';
  const b = mhzStr.slice(-6, -3);
  const c = mhzStr.slice(-3);
  return `${a}.${b}.${c}`;
}
