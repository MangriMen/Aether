import { createRoot } from 'solid-js';
import { beforeEach, describe, expect, it, vi } from 'vitest';

type Warning = { code: string; payload?: unknown };

const ipc = vi.hoisted(() => ({
  deliver: undefined as
    | ((e: { payload: { seq: number; warning: Warning } }) => void)
    | undefined,
  registered: undefined as (() => void) | undefined,
  take: vi.fn(),
  showToast: vi.fn(),
  logError: vi.fn(),
}));

vi.mock('@/platform/ipc', () => ({
  eventsEvents: { warningEventDto: {} },
  processCommands: { takeSkippedHookNotices: ipc.take },
  useReadyEventListener: (_: unknown, cb: typeof ipc.deliver) => {
    ipc.deliver = cb;

    return new Promise<void>((resolve) => {
      ipc.registered = resolve;
    });
  },
}));
vi.mock('@/features/errors', () => ({
  ErrorDetails: () => null,
  getReasonDetails: () => undefined,
  getTranslatedWarning: (w: Warning) => `translated:${w.code}`,
}));
vi.mock('@/shared/i18n', () => ({
  useTranslation: () => [{ t: (key: string) => key }],
}));
vi.mock('@/shared/log', () => ({
  logDebug: vi.fn(),
  logError: ipc.logError,
}));
vi.mock('@/shared/ui-kit', () => ({ showToast: ipc.showToast }));

const flushed = () => new Promise((r) => setTimeout(r, 0));

const skipped: Warning = {
  code: 'POST_EXIT_HOOK_SKIPPED',
  payload: { instanceId: 'a', instanceName: 'A' },
};

// The listener remembers per module that it took the notices, so each test gets a fresh one.
const load = async () => {
  vi.resetModules();
  const { useWarningEventsListener } =
    await import('../useWarningEventsListener');

  return () => createRoot(() => useWarningEventsListener());
};

describe('useWarningEventsListener', () => {
  beforeEach(() => {
    ipc.take.mockReset();
    ipc.showToast.mockReset();
    ipc.logError.mockReset();
  });

  it('should_take_the_notices_only_after_the_listener_is_registered', async () => {
    ipc.take.mockResolvedValue([]);
    const mount = await load();
    mount();
    await flushed();

    expect(ipc.take).not.toHaveBeenCalled();

    ipc.registered?.();
    await flushed();

    expect(ipc.take).toHaveBeenCalledOnce();
  });

  it('should_show_each_taken_notice_as_a_persistent_toast', async () => {
    ipc.take.mockResolvedValue([skipped, skipped]);
    const mount = await load();
    mount();
    ipc.registered?.();
    await flushed();

    expect(ipc.showToast).toHaveBeenCalledTimes(2);
    expect(ipc.showToast).toHaveBeenCalledWith(
      expect.objectContaining({
        description: 'translated:POST_EXIT_HOOK_SKIPPED',
        persistent: true,
        variant: 'warning',
      }),
    );
  });

  it('should_take_the_notices_once_when_the_listener_is_mounted_again', async () => {
    ipc.take.mockResolvedValue([]);
    const mount = await load();
    mount();
    ipc.registered?.();
    await flushed();
    mount();
    ipc.registered?.();
    await flushed();

    expect(ipc.take).toHaveBeenCalledOnce();
  });

  it('should_log_and_not_throw_when_taking_the_notices_fails', async () => {
    ipc.take.mockRejectedValue(new Error('no ipc'));
    const mount = await load();
    mount();
    ipc.registered?.();
    await flushed();

    expect(ipc.logError).toHaveBeenCalled();
    expect(ipc.showToast).not.toHaveBeenCalled();
  });

  it('should_show_a_live_skipped_hook_warning_as_a_persistent_toast', async () => {
    ipc.take.mockResolvedValue([]);
    const mount = await load();
    mount();
    ipc.deliver?.({ payload: { seq: 1, warning: skipped } });

    expect(ipc.showToast).toHaveBeenCalledWith(
      expect.objectContaining({ persistent: true }),
    );
  });

  it('should_not_make_other_warnings_persistent_when_they_arrive_live', async () => {
    ipc.take.mockResolvedValue([]);
    const mount = await load();
    mount();
    ipc.deliver?.({
      payload: { seq: 1, warning: { code: 'INSTANCE_ICON_NOT_SUPPORTED' } },
    });

    expect(ipc.showToast).toHaveBeenCalledWith(
      expect.objectContaining({ persistent: false }),
    );
  });
});
