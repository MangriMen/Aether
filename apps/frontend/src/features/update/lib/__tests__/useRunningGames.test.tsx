import { createRoot } from 'solid-js';
import { describe, expect, it, vi } from 'vitest';

vi.mock('@/features/instance', async (importActual) => ({
  ...(await importActual<Record<string, unknown>>()),
  useInstances: () => ({ data: [{ id: 'a', name: 'Alpha' }] }),
}));

import { RunningInstancesContext } from '@/features/instance';

import { useRunningGames } from '../useRunningGames';

describe('useRunningGames', () => {
  it('should_see_the_games_of_the_provider_it_is_under', () => {
    const value = [
      { instances: { a: { isLoading: false, isRunning: true } } },
      {},
    ] as unknown as Parameters<
      typeof RunningInstancesContext.Provider
    >[0]['value'];

    const games = createRoot((dispose) => {
      let result!: ReturnType<typeof useRunningGames>;
      RunningInstancesContext.Provider({
        value,
        get children() {
          result = useRunningGames();
          return undefined;
        },
      });
      const out = { names: result.names(), isRunning: result.isRunning() };
      dispose();
      return out;
    });

    expect(games).toEqual({ names: ['Alpha'], isRunning: true });
  });
});
