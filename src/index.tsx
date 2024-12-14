import { getCurrentWindow } from '@tauri-apps/api/window';
import { render } from 'solid-js/web';

import App from './app/app';
render(() => <App />, document.getElementById('root') as HTMLElement);
getCurrentWindow().show();
