import { createSignal } from 'solid-js';
import type { Settings } from './settings';
import { getMaxRam, getSettings } from '../api';

const [settings, setSettings] = createSignal<Settings | undefined>();
const [maxRam, setMaxRam] = createSignal(0);

const fetchSettings = async () => {
  try {
    setSettings(await getSettings());
    setMaxRam(await getMaxRam());
  } catch {
    console.error("Can't get settings");
  }
};

export const initializeSettings = fetchSettings;

export const refetchSettings = fetchSettings;

export const useSettings = settings;

export const useMaxRam = maxRam;
