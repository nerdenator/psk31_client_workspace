/** Menu event handling - dispatches Tauri menu events to UI actions */

import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { setTheme } from '../components/theme-toggle';
import { openSettingsDialog } from '../components/settings-dialog';
import type { MenuEvent } from '../types';

let unlisten: UnlistenFn | null = null;

export async function setupMenuEvents(): Promise<void> {
  // Clean up previous listener (prevents leaks during hot reload)
  if (unlisten) {
    unlisten();
    unlisten = null;
  }

  unlisten = await listen<MenuEvent>('menu-event', (event) => {
    const { id } = event.payload;
    console.log(`Menu event: ${id}`);

    switch (id) {
      case 'settings':
        openSettingsDialog('general');
        break;

      case 'config_default':
        openSettingsDialog('general');
        break;

      case 'config_save':
        openSettingsDialog('general');
        break;

      case 'config_delete':
        openSettingsDialog('general');
        break;

      case 'theme_light':
        setTheme('light');
        break;

      case 'theme_dark':
        setTheme('dark');
        break;

      case 'waterfall_colors':
        openSettingsDialog('general');
        break;

      case 'zoom_in':
        console.log('Zoom in');
        break;

      case 'zoom_out':
        console.log('Zoom out');
        break;

      case 'zoom_reset':
        console.log('Zoom reset');
        break;

      case 'documentation':
        window.open('https://github.com/nerdenator/psk31_client_workspace', '_blank');
        break;

      case 'about':
        alert('PSK-31 Client v0.1.0\n\nA cross-platform desktop application for PSK-31 ham radio communication.');
        break;

      default:
        console.log(`Unhandled menu event: ${id}`);
    }
  });
}
