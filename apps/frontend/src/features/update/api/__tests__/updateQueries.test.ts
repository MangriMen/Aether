import { describe, expect, it, vi } from 'vitest';

const errors = vi.hoisted(() => ({ showError: vi.fn() }));
const query = vi.hoisted(() => ({
  options: undefined as { onError: (err: unknown) => void } | undefined,
}));

vi.mock('@tanstack/solid-query', () => ({
  useQuery: vi.fn(),
  useMutation: (options: () => { onError: (err: unknown) => void }) => {
    query.options = options();
  },
}));
vi.mock('@/features/errors', () => errors);
vi.mock('@/platform/ipc', () => ({ updateCommands: {} }));
vi.mock('@/shared/i18n', () => ({
  useTranslation: () => [{ t: (key: string) => key }],
}));

import { useInstallUpdate } from '../updateQueries';

describe('useInstallUpdate', () => {
  it('should_show_the_translated_error_when_the_install_fails', () => {
    const err = { type: 'update', payload: { code: 'NETWORK' } };

    useInstallUpdate();
    query.options!.onError(err);

    expect(errors.showError).toHaveBeenCalledWith(
      expect.objectContaining({ title: 'update.updateError', err }),
    );
  });
});
