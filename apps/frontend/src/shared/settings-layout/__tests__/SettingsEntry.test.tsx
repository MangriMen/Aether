import { render } from 'solid-js/web';
import { afterEach, describe, expect, it } from 'vitest';

import { Switch, SwitchControl, SwitchThumb } from '@/shared/ui-kit';

import { SettingsEntry, useSettingsEntryLabel } from '../SettingsEntry';

let dispose: (() => void) | undefined;

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

const accessibleName = (element: Element): string =>
  (element.getAttribute('aria-labelledby') ?? '')
    .split(/\s+/)
    .filter(Boolean)
    .map((id) => document.getElementById(id)?.textContent ?? '')
    .join(' ')
    .trim();

describe('SettingsEntry', () => {
  it('should_name_the_switch_after_the_entry_title_when_rendered_inside_a_settings_entry', () => {
    const container = document.createElement('div');
    document.body.append(container);

    dispose = render(
      () => (
        <SettingsEntry title='Disable animations'>
          <Switch>
            <SwitchControl {...useSettingsEntryLabel()}>
              <SwitchThumb />
            </SwitchControl>
          </Switch>
        </SettingsEntry>
      ),
      container,
    );

    const control = container.querySelector('[role="switch"]');

    expect(control).not.toBeNull();
    expect(accessibleName(control as Element)).toBe('Disable animations');
  });
});
