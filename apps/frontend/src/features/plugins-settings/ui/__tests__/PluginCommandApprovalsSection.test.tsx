import type { JSX } from 'solid-js';

import { render } from 'solid-js/web';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type { CommandApproval } from '@/features/plugins';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

import { PluginCommandApprovalsSection } from '../PluginCommandApprovalsSection';

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

  return container;
};

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

const approval = (
  overrides: Partial<CommandApproval> = {},
): CommandApproval => ({
  jarSha256: 'a'.repeat(64),
  jvmOptions: [],
  jar: '/cache/downloads/sha256/abc',
  approvedAt: '2026-11-16T10:00:00Z',
  ...overrides,
});

describe('PluginCommandApprovalsSection', () => {
  it('should_list_each_program_with_a_short_hash', () => {
    const container = mount(() => (
      <PluginCommandApprovalsSection
        approvals={[approval(), approval({ jar: '/cache/other.jar' })]}
        onRevoke={vi.fn()}
      />
    ));

    expect(container.querySelectorAll('li')).toHaveLength(2);
    expect(container.textContent).toContain('/cache/downloads/sha256/abc');
    expect(container.textContent).toContain(`${'a'.repeat(12)}…`);
    expect(container.textContent).not.toContain('a'.repeat(13));
  });

  it('should_show_the_jvm_options_when_the_approval_has_some', () => {
    const container = mount(() => (
      <PluginCommandApprovalsSection
        approvals={[approval({ jvmOptions: ['-Xmx2G'] })]}
        onRevoke={vi.fn()}
      />
    ));

    expect(container.textContent).toContain('-Xmx2G');
  });

  it('should_hand_over_the_approval_of_the_row_when_it_is_taken_back', () => {
    const onRevoke = vi.fn();
    const second = approval({
      jar: '/cache/other.jar',
      jarSha256: 'b'.repeat(64),
    });
    const container = mount(() => (
      <PluginCommandApprovalsSection
        approvals={[approval(), second]}
        onRevoke={onRevoke}
      />
    ));

    container.querySelectorAll('button')[1]!.click();

    expect(onRevoke).toHaveBeenCalledWith(second);
  });

  it('should_not_let_an_approval_be_taken_back_while_disabled', () => {
    const onRevoke = vi.fn();
    const container = mount(() => (
      <PluginCommandApprovalsSection
        approvals={[approval()]}
        onRevoke={onRevoke}
        disabled
      />
    ));

    container.querySelector('button')!.click();

    expect(onRevoke).not.toHaveBeenCalled();
  });
});
