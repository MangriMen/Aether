import type { JSX } from 'solid-js';

import { render } from 'solid-js/web';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

import type { PendingCommandApproval } from '../../model/commandApproval';

const state = vi.hoisted(() => ({
  approvals: [] as unknown[],
  answer: vi.fn(),
  pending: false,
}));

vi.mock('../../api/commandApprovalQueries', () => ({
  useCommandApprovals: () => ({ data: state.approvals }),
  useAnswerCommandApproval: () => ({
    mutate: state.answer,
    get isPending() {
      return state.pending;
    },
  }),
}));

const { CommandApprovalDialog } = await import('../CommandApprovalDialog');

const approval = (
  overrides: Partial<PendingCommandApproval> = {},
): PendingCommandApproval => ({
  request_id: 'r1',
  plugin_id: 'packwiz',
  java: 'C:/java/bin/java.exe',
  current_dir: 'C:/cache',
  options: [],
  target: '#/cache/a.jar',
  rest: ['--token', 'secret-token'],
  jar: { plugin_path: '/cache/a.jar', sha256: 'abc', size: 100 },
  once_only: null,
  expires_at: new Date(Date.now() + 300_000).toISOString(),
  ...overrides,
});

let dispose: (() => void) | undefined;

const mount = (ui: () => JSX.Element) => {
  const container = document.createElement('div');
  document.body.append(container);
  dispose = render(
    () => (
      <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
        {ui()}
      </I18nProvider>
    ),
    container,
  );
};

const button = (label: string) =>
  [...document.querySelectorAll('button')].find((b) =>
    b.textContent?.includes(label),
  );

describe('CommandApprovalDialog', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    state.answer.mockReset();
    state.pending = false;
    state.approvals = [approval()];
  });

  afterEach(() => {
    dispose?.();
    dispose = undefined;
    document.body.innerHTML = '';
    vi.useRealTimers();
  });

  it('should_keep_both_allows_out_of_reach_right_after_a_question_shows', () => {
    mount(() => <CommandApprovalDialog />);

    expect(button('Allow once')?.disabled).toBe(true);
    expect(button('Always allow')?.disabled).toBe(true);
    expect(button('Deny')?.disabled).toBe(false);

    vi.advanceTimersByTime(800);

    expect(button('Allow once')?.disabled).toBe(false);
    expect(button('Always allow')?.disabled).toBe(false);
  });

  it('should_send_the_answer_of_the_button_for_the_question_shown', () => {
    mount(() => <CommandApprovalDialog />);
    vi.advanceTimersByTime(800);

    button('Allow once')!.click();

    expect(state.answer).toHaveBeenCalledWith({
      requestId: 'r1',
      answer: 'once',
    });
  });

  it('should_disable_every_button_while_an_answer_is_on_its_way', () => {
    state.pending = true;
    mount(() => <CommandApprovalDialog />);
    vi.advanceTimersByTime(800);

    expect(button('Deny')?.disabled).toBe(true);
    expect(button('Allow once')?.disabled).toBe(true);
  });

  it('should_not_offer_always_when_only_once_is_allowed', () => {
    state.approvals = [approval({ once_only: { code: 'JAR_HAS_CLASS_PATH' } })];
    mount(() => <CommandApprovalDialog />);
    vi.advanceTimersByTime(800);

    expect(button('Always allow')?.disabled).toBe(true);
    expect(button('Allow once')?.disabled).toBe(false);
  });

  it('should_keep_an_option_value_out_of_the_open_text', () => {
    state.approvals = [
      approval({
        options: [],
        target: null,
        rest: ['-Dapi.token=abc123', '-jar', '#/cache/a.jar'],
        once_only: { code: 'UNLISTED_OPTION', option: '-Dapi.token=abc123' },
      }),
    ];
    mount(() => <CommandApprovalDialog />);

    const details = document.querySelector('details');
    const text = document.body.textContent ?? '';
    const outside = text.replace(details?.textContent ?? '', '');

    expect(outside).not.toContain('abc123');
  });
});
