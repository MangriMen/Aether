import { createSignal } from 'solid-js';
import { listPlugins } from '../api';
import type { PluginMetadata } from './plugin';

const [plugins, setPlugins] = createSignal<PluginMetadata[]>([]);

const fetchPlugins = async () => {
  try {
    setPlugins(await listPlugins());
  } catch {
    console.error("Can't get plugins");
  }
};

export const initializePlugins = fetchPlugins;

export const usePlugins = () => plugins;

export const refetchPlugins = fetchPlugins;
