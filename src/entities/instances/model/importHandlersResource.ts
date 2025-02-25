import { createSignal } from 'solid-js';
import { getImportHandlers } from '../api';
import type { ImportHandler } from './instance';

const [importHandlers, setImportHandlers] = createSignal<ImportHandler[]>([]);

const fetchImportHandlers = async () => {
  try {
    setImportHandlers(await getImportHandlers());
  } catch (e) {
    console.error("Can't get import handlers", e);
  }
};

export const initializeImportHandlers = fetchImportHandlers;

export const useImportHandlers = () => importHandlers;
