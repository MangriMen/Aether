import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { TFunction } from '@/shared/i18n';

const showToast = vi.hoisted(() => vi.fn());

vi.mock('@/shared/ui-kit', () => ({ showToast }));
vi.mock('@/shared/log', () => ({ logError: vi.fn() }));

import { showError } from '../showError';

const t = ((key: string) => key) as unknown as TFunction;

/** A launcher error that has details (the plugin's own words) to show. */
const refused = {
  type: 'plugin',
  payload: {
    code: 'HOST_NOT_ALLOWED',
    payload: {
      plugin_id: 'packwiz',
      host: 'cdn.example',
      cause: { type: 'unknown', details: 'cdn refused' },
    },
  },
} as unknown as Error;

const toastAction = (): unknown => {
  const [params] = showToast.mock.calls[0] as [{ action?: unknown }];

  return params.action;
};

describe('showError', () => {
  beforeEach(() => {
    showToast.mockClear();
  });

  it('should still show the details when the caller passes `action: undefined`', () => {
    showError({ err: refused, t, action: undefined });

    expect(typeof toastAction()).toBe('function');
  });

  it('should use the action of the caller when it passes one', () => {
    const own = () => 'own';

    showError({ err: refused, t, action: own });

    expect(toastAction()).toBe(own);
  });
});
