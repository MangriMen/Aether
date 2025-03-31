import { createSignal } from 'solid-js';
import type { Settings } from './settings';
import { getSettings } from '../api';

const [settings, setSettings] = createSignal<Settings | undefined>();

const fetchSettings = async () => {
  try {
    setSettings(await getSettings());
  } catch {
    console.error("Can't get settings");
  }
};

export const initializeSettings = fetchSettings;

export const refetchSettings = fetchSettings;

export const useSettings = settings;
