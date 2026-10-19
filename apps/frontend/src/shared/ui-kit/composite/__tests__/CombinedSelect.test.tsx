import type { JSX } from 'solid-js';

import { render } from 'solid-js/web';
import { afterEach, describe, expect, it } from 'vitest';

import { SelectItem } from '../../primitives/Select';
import { CombinedSelect } from '../CombinedSelect';

type Named = { id: number; name: string };

const NAMED: Named[] = [
  { id: 1, name: 'Alpha' },
  { id: 2, name: 'Beta' },
];

let dispose: (() => void) | undefined;

const mount = (ui: () => JSX.Element) => {
  dispose = render(ui, document.body);
};

const trigger = () => document.querySelector<HTMLButtonElement>('button')!;

const triggerText = () => trigger().textContent ?? '';

const open = () => {
  trigger().dispatchEvent(
    new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }),
  );
  trigger().click();
};

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('CombinedSelect selected value', () => {
  it('shows the property named by a string optionTextValue', () => {
    mount(() => (
      <CombinedSelect<Named, never>
        options={NAMED}
        optionValue='id'
        optionTextValue='name'
        value={NAMED[1]}
      />
    ));

    expect(triggerText()).toContain('Beta');
  });

  it('shows the result of a function optionTextValue', () => {
    mount(() => (
      <CombinedSelect<Named, never>
        options={NAMED}
        optionValue='id'
        optionTextValue={(option) => `#${option.id} ${option.name}`}
        value={NAMED[0]}
      />
    ));

    expect(triggerText()).toContain('#1 Alpha');
  });

  it('shows a primitive string option', () => {
    mount(() => (
      <CombinedSelect<string, never> options={['one', 'two']} value='two' />
    ));

    expect(triggerText()).toContain('two');
  });

  it('shows the falsy number 0', () => {
    mount(() => (
      <CombinedSelect<number, never> options={[0, 1, 2]} value={0} />
    ));

    expect(triggerText()).toContain('0');
  });

  it('shows the falsy boolean false', () => {
    mount(() => (
      <CombinedSelect<boolean, never> options={[true, false]} value={false} />
    ));

    expect(triggerText()).toContain('false');
  });

  it('does not crash on an empty string option', () => {
    mount(() => <CombinedSelect<string, never> options={['', 'a']} value='' />);

    expect(trigger()).not.toBeNull();
  });

  it('renders no text when nothing is selected', () => {
    mount(() => (
      <CombinedSelect<Named, never>
        options={NAMED}
        optionValue='id'
        optionTextValue='name'
      />
    ));

    expect(triggerText().trim()).toBe('');
  });
});

describe('CombinedSelect props', () => {
  it('puts the class prop on the root element', () => {
    mount(() => (
      <CombinedSelect<string, never>
        class='my-root-class'
        options={['a']}
        value='a'
      />
    ));

    const root = document.querySelector<HTMLElement>('.my-root-class');

    expect(root).not.toBeNull();
    expect(root!.contains(trigger())).toBe(true);
  });

  it('renders textValue in the default item', () => {
    mount(() => (
      <CombinedSelect<Named, never>
        options={NAMED}
        optionValue='id'
        optionTextValue='name'
      />
    ));

    open();

    const items = [...document.body.querySelectorAll('[role="option"]')];

    expect(items.map((item) => item.textContent)).toEqual(['Alpha', 'Beta']);
  });

  it('uses a custom itemComponent when provided', () => {
    mount(() => (
      <CombinedSelect<Named, never>
        options={NAMED}
        optionValue='id'
        optionTextValue='name'
        itemComponent={(props) => (
          <SelectItem item={props.item}>
            custom-{props.item.textValue}
          </SelectItem>
        )}
      />
    ));

    open();

    const items = [...document.body.querySelectorAll('[role="option"]')];

    expect(items.map((item) => item.textContent)).toEqual([
      'custom-Alpha',
      'custom-Beta',
    ]);
  });
});
