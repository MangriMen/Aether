import { onCleanup, onMount } from 'solid-js';
import { isDeveloperMode, setIsDeveloperMode } from '../model';

export const useDeveloperModeCounter = () => {
  let counter = 0;

  onMount(() => {
    counter = 0;
  });

  onCleanup(() => {
    counter = 0;
  });

  return () => {
    counter++;
    if (counter >= 5) {
      setIsDeveloperMode(!isDeveloperMode());
      counter = 0;
    }
  };
};
