import { getCurrentWindow } from '@tauri-apps/api/window';

export const showWindow = () => getCurrentWindow().show();
