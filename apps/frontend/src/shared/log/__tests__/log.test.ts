import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const errorMock = vi.fn<(message: string) => Promise<void>>();

vi.mock('@tauri-apps/plugin-log', () => ({
  error: (message: string) => errorMock(message),
}));

const load = async () => {
  vi.resetModules();
  return import('../log');
};

describe('formatLogData', () => {
  it('formats an Error as name: message with at most 5 stack frames', async () => {
    const { formatLogData } = await load();
    const err = new TypeError('boom');
    err.stack = [
      'TypeError: boom',
      ...Array.from({ length: 9 }, (_, i) => `    at fn${i} (file.ts:1:1)`),
    ].join('\n');

    const lines = formatLogData([err]).split('\n');

    expect(lines[0]).toBe('TypeError: boom');
    expect(lines).toHaveLength(6);
  });

  it('formats a launcher-error-like object as type.code', async () => {
    const { formatLogData } = await load();

    expect(
      formatLogData([
        { type: 'instance', payload: { code: 'NOT_FOUND', id: 'secret' } },
      ]),
    ).toBe('instance.NOT_FOUND');
  });

  it('serializes an arbitrary object as JSON', async () => {
    const { formatLogData } = await load();

    expect(formatLogData(['ctx', { a: 1 }])).toBe('ctx {"a":1}');
  });

  it('survives a circular structure', async () => {
    const { formatLogData } = await load();
    const a: Record<string, unknown> = {};
    a.self = a;

    expect(formatLogData([a])).toBe('{"self":"[Circular]"}');
  });

  it('truncates the result to 2000 characters', async () => {
    const { formatLogData } = await load();

    expect(formatLogData(['x'.repeat(5000)])).toHaveLength(2000);
  });
});

describe('logError', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    errorMock.mockReset();
    errorMock.mockResolvedValue(undefined);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('drops an identical message within 10 seconds and allows it afterwards', async () => {
    const { logError } = await load();

    logError('same');
    logError('same');
    vi.advanceTimersByTime(9_000);
    logError('same');
    expect(errorMock).toHaveBeenCalledTimes(1);

    vi.advanceTimersByTime(1_500);
    logError('same');
    expect(errorMock).toHaveBeenCalledTimes(2);
  });

  it('sends at most 20 records per minute', async () => {
    const { logError } = await load();

    for (let i = 0; i < 30; i++) {
      logError(`message ${i}`);
    }

    expect(errorMock).toHaveBeenCalledTimes(20);

    vi.advanceTimersByTime(61_000);
    logError('later');
    expect(errorMock).toHaveBeenCalledTimes(21);
  });

  it('does not throw when the plugin rejects', async () => {
    errorMock.mockRejectedValue(new Error('no tauri'));
    const { logError } = await load();

    expect(() => logError('x')).not.toThrow();
    await vi.advanceTimersByTimeAsync(0);
  });

  it('does not throw when the plugin throws synchronously', async () => {
    errorMock.mockImplementation(() => {
      throw new Error('no ipc');
    });
    const { logError } = await load();

    expect(() => logError('x')).not.toThrow();
  });
});
