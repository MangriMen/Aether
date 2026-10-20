import { describe, expect, it } from 'vitest';

import { pluginsPaneState } from '../pluginsPaneState';

describe('pluginsPaneState', () => {
  it('reports loading while the first request is in flight', () => {
    expect(
      pluginsPaneState({ isError: false, isLoading: true, count: undefined }),
    ).toBe('loading');
  });

  it('reports an error instead of an empty list when the first load fails', () => {
    expect(
      pluginsPaneState({ isError: true, isLoading: false, count: undefined }),
    ).toBe('error');
  });

  it('keeps the list when a refetch fails over existing data', () => {
    expect(
      pluginsPaneState({ isError: true, isLoading: false, count: 2 }),
    ).toBe('list');
  });

  it('reports an empty list when the request succeeded with nothing', () => {
    expect(
      pluginsPaneState({ isError: false, isLoading: false, count: 0 }),
    ).toBe('empty');
  });

  it('reports the list when plugins are present', () => {
    expect(
      pluginsPaneState({ isError: false, isLoading: false, count: 3 }),
    ).toBe('list');
  });
});
