import { createEffect, createMemo, createSignal, on } from 'solid-js';

import { debounce } from '@/shared/utils';

import {
  createLineFilter,
  type LineFilter,
  type LogLevel,
} from '../model/logLines';

const SEARCH_DEBOUNCE_MS = 300;

/**
 * The chosen levels and search text, and the filter they make. `lineFilter` follows every
 * key; `searchFilter` follows it after a pause, for work that must not start at each key,
 * such as a search through a large log.
 */
export const createLogFilter = () => {
  const [levels, setLevels] = createSignal<LogLevel[]>([]);
  const [query, setQuery] = createSignal('');

  const lineFilter = createMemo(() =>
    createLineFilter(new Set(levels()), query()),
  );

  const [searchFilter, setSearchFilter] = createSignal(lineFilter());
  const updateSearchFilter = debounce((filter: LineFilter | undefined) => {
    // The filter is a function, which a signal would call as an updater.
    setSearchFilter(() => filter);
  }, SEARCH_DEBOUNCE_MS);

  createEffect(
    on(lineFilter, (filter) => updateSearchFilter(filter), { defer: true }),
  );

  return { levels, setLevels, query, setQuery, lineFilter, searchFilter };
};
