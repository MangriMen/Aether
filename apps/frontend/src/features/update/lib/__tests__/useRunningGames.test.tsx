import { createRoot } from 'solid-js';
import { createStore, reconcile } from 'solid-js/store';
import { describe, expect, it, vi } from 'vitest';

vi.mock('@/features/instance', async (importActual) => ({
  ...(await importActual<Record<string, unknown>>()),
  useInstances: () => ({ data: [{ id: 'a', name: 'Alpha' }] }),
}));

import { RunningInstancesContext } from '@/features/instance';

import { useRunningGames } from '../useRunningGames';

describe('useRunningGames', () => {
  it('should_follow_the_provider_when_a_game_starts_and_finishes', () => {
    const [state, setState] = createStore<{
      instances: Record<string, { isLoading: boolean; isRunning: boolean }>;
    }>({ instances: {} });
    const value = [state, {}] as unknown as Parameters<
      typeof RunningInstancesContext.Provider
    >[0]['value'];

    createRoot((dispose) => {
      let games!: ReturnType<typeof useRunningGames>;
      RunningInstancesContext.Provider({
        value,
        get children() {
          games = useRunningGames();
          return undefined;
        },
      });

      expect(games.isRunning()).toBe(false);

      setState('instances', 'a', { isLoading: false, isRunning: true });
      expect(games.names()).toEqual(['Alpha']);

      setState('instances', reconcile({}));
      expect(games.isRunning()).toBe(false);

      dispose();
    });
  });
});
