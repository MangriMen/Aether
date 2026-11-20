import { createRoot } from 'solid-js';
import { createStore, reconcile } from 'solid-js/store';
import { describe, expect, it, vi } from 'vitest';

const carryOn = vi.hoisted(() => ({
  data: undefined as boolean | undefined,
  asked: [] as string[][],
}));

vi.mock('@/features/instance', async (importActual) => ({
  ...(await importActual<Record<string, unknown>>()),
  useInstances: () => ({ data: [{ id: 'a', name: 'Alpha' }] }),
}));
vi.mock('../../api/updateQueries', () => ({
  useGamesCarryOnUpdate: (games: () => readonly string[]) => ({
    get data() {
      carryOn.asked.push([...games()]);
      return carryOn.data;
    },
  }),
}));

import { RunningInstancesContext } from '@/features/instance';

import { useRunningGames } from '../useRunningGames';

type State = { isLoading: boolean; isRunning: boolean };

const inProvider = (
  instances: Record<string, State>,
  check: (games: ReturnType<typeof useRunningGames>) => void,
) => {
  const [state] = createStore<{ instances: Record<string, State> }>({
    instances,
  });
  const value = [state, {}] as unknown as Parameters<
    typeof RunningInstancesContext.Provider
  >[0]['value'];

  createRoot((dispose) => {
    RunningInstancesContext.Provider({
      value,
      get children() {
        check(useRunningGames());
        return undefined;
      },
    });
    dispose();
  });
};

describe('useRunningGames', () => {
  it('should_follow_the_provider_when_a_game_starts_and_finishes', () => {
    const [state, setState] = createStore<{
      instances: Record<string, State>;
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

  it('should_not_block_the_update_when_no_game_runs', () => {
    carryOn.data = undefined;

    inProvider({}, (games) => {
      expect(games.isBlocked()).toBe(false);
      expect(games.keepsGames()).toBe(false);
    });
  });

  it('should_block_the_update_under_a_game_until_it_is_known_to_carry_on', () => {
    const running = { a: { isLoading: false, isRunning: true } };

    carryOn.data = undefined;
    inProvider(running, (games) => expect(games.isBlocked()).toBe(true));

    carryOn.data = false;
    inProvider(running, (games) => expect(games.isBlocked()).toBe(true));

    carryOn.data = true;
    inProvider(running, (games) => {
      expect(games.isBlocked()).toBe(false);
      expect(games.keepsGames()).toBe(true);
    });
  });

  it('should_block_the_update_while_a_game_is_launching', () => {
    carryOn.data = true;

    inProvider({ a: { isLoading: true, isRunning: false } }, (games) =>
      expect(games.isBlocked()).toBe(true),
    );
  });
});

describe('useRunningGames carry on question', () => {
  it('should_ask_again_when_a_launching_game_has_started', () => {
    const asked = (state: State) => {
      carryOn.asked = [];
      inProvider({ a: state }, (games) => games.isBlocked());
      return carryOn.asked.at(-1);
    };

    const launching = asked({ isLoading: true, isRunning: false });
    const running = asked({ isLoading: false, isRunning: true });

    expect(launching).not.toEqual(running);
  });
});
