import { beforeEach, describe, expect, it, vi } from 'vitest';

const commands = vi.hoisted(() => ({ listContents: vi.fn() }));

vi.mock('@/platform/ipc', () => ({ instanceCommands: commands }));

import { listContentBatched } from '../listContentBatched';

const file = { fileName: 'a.jar' };

beforeEach(() => {
  commands.listContents.mockReset();
});

describe('listContentBatched', () => {
  it('should_list_the_instances_asked_in_one_turn_with_one_call', async () => {
    commands.listContents.mockResolvedValue({
      a: { 'mods/a.jar': file },
      b: {},
    });

    const [a, b] = await Promise.all([
      listContentBatched('a'),
      listContentBatched('b'),
    ]);

    expect(commands.listContents).toHaveBeenCalledTimes(1);
    expect(commands.listContents).toHaveBeenCalledWith(['a', 'b']);
    expect(a).toEqual({ 'mods/a.jar': file });
    expect(b).toEqual({});
  });

  it('should_fail_only_the_instance_the_backend_left_out', async () => {
    commands.listContents.mockResolvedValue({ a: {} });

    const [a, b] = await Promise.allSettled([
      listContentBatched('a'),
      listContentBatched('b'),
    ]);

    expect(a.status).toBe('fulfilled');
    expect(b.status).toBe('rejected');
  });

  it('should_fail_every_instance_when_the_call_fails', async () => {
    commands.listContents.mockRejectedValue(new Error('offline'));

    const results = await Promise.allSettled([
      listContentBatched('a'),
      listContentBatched('b'),
    ]);

    expect(results.map((r) => r.status)).toEqual(['rejected', 'rejected']);
  });

  it('should_ask_again_with_a_new_call_in_a_later_turn', async () => {
    commands.listContents.mockResolvedValue({ a: {} });

    await listContentBatched('a');
    await listContentBatched('a');

    expect(commands.listContents).toHaveBeenCalledTimes(2);
  });
});
