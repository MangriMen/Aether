import { createSignal } from 'solid-js';
import { getImportConfigs } from '../api';
import type { ImportHandler } from './instance';

const [importConfigs, setImportConfigs] = createSignal<ImportHandler[]>([]);

const fetchImportConfigs = async () => {
  try {
    setImportConfigs(await getImportConfigs());
  } catch (e) {
    console.error("Can't get import handlers", e);
  }
};

export const initializeImportConfigs = fetchImportConfigs;

export const refetchImportConfigs = fetchImportConfigs;

export const useImportConfigs = () => importConfigs;
