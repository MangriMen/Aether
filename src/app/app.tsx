import { getCurrentWindow } from '@tauri-apps/api/window';
import { onMount } from 'solid-js';

import './app.css';
import { AppRouter } from './routes';

function App() {
  onMount(() => {
    getCurrentWindow().show();
  });

  return <AppRouter />;
}

export default App;
