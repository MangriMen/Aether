import './app.css';
import { AppRouter } from './routes';
import { WindowObserver } from './ui';

function App() {
  return (
    <>
      <WindowObserver />
      <AppRouter />
    </>
  );
}

export default App;
