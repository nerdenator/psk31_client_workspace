import { test, expect } from '@playwright/test';
import { mockInvoke, fireEvent } from './helpers';

/**
 * Phase 6 E2E tests for error handling, status bar, and the full application flow.
 *
 * Tests backend-initiated error events (serial disconnect, audio hot-plug),
 * status bar hydration on load, and a composed end-to-end smoke test.
 */

/** Minimal mocks for tests that don't need serial/audio devices */
const DISCONNECTED_STATUS = {
  get_connection_status: {
    serial_connected: false,
    serial_port: null,
    audio_streaming: false,
    audio_device: null,
  },
  list_audio_devices: [],
  list_serial_ports: [],
};

test.describe('Status Bar', () => {
  test('shows disconnected indicators on page load', async ({ page }) => {
    await mockInvoke(page, DISCONNECTED_STATUS);
    await page.goto('/');

    await expect(page.locator('#statusbar-serial .status-dot')).toHaveClass(/disconnected/);
    await expect(page.locator('#statusbar-serial .status-text')).toHaveText('CAT');
    await expect(page.locator('#statusbar-audio .status-dot')).toHaveClass(/disconnected/);
    await expect(page.locator('#statusbar-audio .status-text')).toHaveText('Audio');
  });

  test('shows connected serial indicator when seeded from backend', async ({ page }) => {
    await mockInvoke(page, {
      get_connection_status: {
        serial_connected: true,
        serial_port: '/dev/cu.usbserial-1420',
        audio_streaming: false,
        audio_device: null,
      },
      list_audio_devices: [],
      list_serial_ports: [],
    });
    await page.goto('/');

    await expect(page.locator('#statusbar-serial .status-dot')).toHaveClass(/connected/);
    // Port name is truncated to 18 chars; assert partial match
    await expect(page.locator('#statusbar-serial .status-text')).toContainText('usbserial');
  });

  test('signal-level event activates correct number of bars', async ({ page }) => {
    await mockInvoke(page, DISCONNECTED_STATUS);
    await page.goto('/');

    // level=0.6 → Math.round(0.6 * 5) = 3 active bars
    await fireEvent(page, 'signal-level', { level: 0.6 });

    const bars = page.locator('.signal-bars .signal-bar');
    await expect(bars).toHaveCount(5);
    await expect(bars.nth(0)).toHaveClass(/active/);
    await expect(bars.nth(1)).toHaveClass(/active/);
    await expect(bars.nth(2)).toHaveClass(/active/);
    await expect(bars.nth(3)).not.toHaveClass(/active/);
    await expect(bars.nth(4)).not.toHaveClass(/active/);
  });
});

test.describe('Error Toasts', () => {
  test('serial-disconnected event shows error toast with port name', async ({ page }) => {
    await mockInvoke(page, DISCONNECTED_STATUS);
    await page.goto('/');

    await fireEvent(page, 'serial-disconnected', {
      reason: 'Device disconnected',
      port: '/dev/cu.usbserial-1420',
    });

    await expect(page.locator('.toast.toast-error')).toContainText(
      'CAT disconnected: /dev/cu.usbserial-1420',
    );
  });

  test('serial-disconnected event resets serial panel to N/C', async ({ page }) => {
    await mockInvoke(page, {
      get_connection_status: {
        serial_connected: false,
        serial_port: null,
        audio_streaming: false,
        audio_device: null,
      },
      list_serial_ports: [{ name: '/dev/cu.usbserial-1420', port_type: 'USB (10C4:EA60)' }],
      connect_serial: {
        port: '/dev/cu.usbserial-1420',
        baud_rate: 38400,
        frequency_hz: 14070000,
        mode: 'DATA-USB',
        connected: true,
      },
      list_audio_devices: [],
    });
    await page.goto('/');

    // Connect serial so the panel is in connected state
    await page.locator('#serial-port').selectOption('/dev/cu.usbserial-1420');
    await page.locator('#serial-connect-btn').click();
    await expect(page.locator('#cat-status .status-dot')).toHaveClass(/connected/);

    // Backend fires a disconnect event
    await fireEvent(page, 'serial-disconnected', {
      reason: 'Device lost',
      port: '/dev/cu.usbserial-1420',
    });

    // Panel should reset
    await expect(page.locator('#cat-status .status-text')).toHaveText('N/C');
    await expect(page.locator('#cat-status .status-dot')).toHaveClass(/disconnected/);
    await expect(page.locator('#serial-connect-btn')).toHaveText('Connect');
  });

  test('audio-status error event shows error toast', async ({ page }) => {
    await mockInvoke(page, DISCONNECTED_STATUS);
    await page.goto('/');

    await fireEvent(page, 'audio-status', { status: 'error: audio device lost' });

    await expect(page.locator('.toast.toast-error')).toContainText('Audio device lost');
  });

  test('audio-status error event resets audio panel', async ({ page }) => {
    await mockInvoke(page, {
      get_connection_status: {
        serial_connected: false,
        serial_port: null,
        audio_streaming: false,
        audio_device: null,
      },
      list_audio_devices: [
        { id: 'mic-1', name: 'Built-in Microphone', is_input: true, is_default: true },
      ],
      start_audio_stream: null,
      list_serial_ports: [],
    });
    await page.goto('/');

    // Start audio so the panel is in streaming state
    await page.locator('#audio-input').selectOption('mic-1');
    await expect(page.locator('#audio-in-status .status-dot')).toHaveClass(/connected/);

    // Backend fires an audio error event
    await fireEvent(page, 'audio-status', { status: 'error: audio device lost' });

    // Panel should reset
    await expect(page.locator('#audio-in-status .status-text')).toHaveText('N/C');
    await expect(page.locator('#audio-in-status .status-dot')).toHaveClass(/disconnected/);
    await expect(page.locator('#audio-input')).toHaveValue('');
  });
});

test.describe('Full Application Flow', () => {
  test('connect serial → start audio → receive text → transmit → disconnect', async ({ page }) => {
    await mockInvoke(page, {
      get_connection_status: {
        serial_connected: false,
        serial_port: null,
        audio_streaming: false,
        audio_device: null,
      },
      list_serial_ports: [{ name: '/dev/cu.usbserial-1420', port_type: 'USB (10C4:EA60)' }],
      connect_serial: {
        port: '/dev/cu.usbserial-1420',
        baud_rate: 38400,
        frequency_hz: 14070000,
        mode: 'DATA-USB',
        connected: true,
      },
      disconnect_serial: null,
      list_audio_devices: [
        { id: 'mic-1', name: 'FT-991A USB Audio CODEC', is_input: true, is_default: false },
      ],
      start_audio_stream: null,
      stop_audio_stream: null,
      start_rx: null,
      start_tx: null,
      stop_tx: null,
    });

    // When start_tx is called, fire a tx-status:complete event after a short delay
    // (mirrors the pattern in app.spec.ts)
    await page.addInitScript(() => {
      const orig = (window as any).__TAURI_INTERNALS__.invoke;
      (window as any).__TAURI_INTERNALS__.invoke = (cmd: string, args?: any) => {
        if (cmd === 'start_tx') {
          setTimeout(() => {
            (window as any).__dispatchTauriEvent__('tx-status', {
              status: 'complete',
              progress: 1.0,
            });
          }, 200);
        }
        return orig(cmd, args);
      };
    });

    await page.goto('/');

    // ── Step 1: Connect serial ──────────────────────────────────────────────
    await page.locator('#serial-port').selectOption('/dev/cu.usbserial-1420');
    await page.locator('#serial-connect-btn').click();

    // Frequency and mode update from the connect response
    await expect(page.locator('.frequency-value').first()).toHaveText('14.070.000');
    await expect(page.locator('.frequency-mode')).toHaveText('DATA-USB');

    // Status bar serial indicator shows connected
    await expect(page.locator('#statusbar-serial .status-dot')).toHaveClass(/connected/);

    // ── Step 2: Start audio ─────────────────────────────────────────────────
    await page.locator('#audio-input').selectOption('mic-1');
    await expect(page.locator('#audio-in-status .status-dot')).toHaveClass(/connected/);
    await expect(page.locator('#statusbar-audio .status-dot')).toHaveClass(/connected/);

    // ── Step 3: Receive text (simulated via DOM — real event path tested in rx.spec.ts) ──
    await page.evaluate(() => {
      document.getElementById('rx-content')!.textContent = 'CQ CQ DE W1AW';
    });
    await expect(page.locator('#rx-content')).toHaveText('CQ CQ DE W1AW');

    // ── Step 4: Transmit ────────────────────────────────────────────────────
    // Add audio output device so TX path can proceed
    await page.evaluate(() => {
      const select = document.getElementById('audio-output') as HTMLSelectElement;
      const opt = document.createElement('option');
      opt.value = 'test-speaker';
      opt.textContent = 'Test Speaker';
      select.appendChild(opt);
      select.value = 'test-speaker';
    });

    await page.locator('#tx-input').fill('73 DE KD9ABC');
    await page.locator('.tx-btn-send').click();

    // UI switches to TX immediately
    await expect(page.locator('.ptt-indicator')).toHaveText('TX');

    // tx-status:complete event fires at 200ms → UI returns to RX
    await expect(page.locator('.ptt-indicator')).toHaveText('RX', { timeout: 3000 });

    // ── Step 5: Disconnect serial ───────────────────────────────────────────
    // Connect button flashes for 10s before showing "Disconnect"
    await expect(page.locator('#serial-connect-btn')).toHaveText('Disconnect', {
      timeout: 12000,
    });
    await page.locator('#serial-connect-btn').click();

    await expect(page.locator('#cat-status .status-text')).toHaveText('N/C');
    await expect(page.locator('#serial-connect-btn')).toHaveText('Connect');
  });
});
