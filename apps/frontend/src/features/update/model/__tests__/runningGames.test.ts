import { describe, expect, it } from 'vitest';

import { getRunningGameNames } from '../runningGames';

const instances = [
  { id: 'a', name: 'Alpha' },
  { id: 'b', name: 'Beta' },
];

describe('getRunningGameNames', () => {
  it('should_name_only_the_games_that_run', () => {
    expect(
      getRunningGameNames(
        {
          a: { isRunning: true },
          b: { isRunning: false },
          c: undefined,
        },
        instances,
      ),
    ).toEqual(['Alpha']);
  });

  it('should_be_empty_when_nothing_runs_so_the_update_is_free', () => {
    expect(getRunningGameNames({}, instances)).toEqual([]);
  });

  it('should_fall_back_to_the_id_when_the_instance_is_not_listed', () => {
    expect(getRunningGameNames({ x: { isRunning: true } }, undefined)).toEqual([
      'x',
    ]);
  });
});
