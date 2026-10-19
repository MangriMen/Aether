import type { JSX } from 'solid-js';

import { render } from 'solid-js/web';
import { afterEach, describe, expect, it } from 'vitest';

import { TitledBlock } from '../TitledBlock';

let dispose: (() => void) | undefined;

const mount = (ui: () => JSX.Element) => {
  dispose = render(ui, document.body);
};

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('TitledBlock', () => {
  it('renders the title as a heading', () => {
    mount(() => <TitledBlock title='Settings'>content</TitledBlock>);

    const heading = document.querySelector('h2');

    expect(heading).not.toBeNull();
    expect(heading!.textContent).toBe('Settings');
  });

  it('omits the heading when no title is given', () => {
    mount(() => <TitledBlock>content</TitledBlock>);

    expect(document.querySelector('h2')).toBeNull();
    expect(document.body.textContent).toContain('content');
  });

  it('renders children next to the title', () => {
    mount(() => (
      <TitledBlock title='Settings'>
        <span>child</span>
      </TitledBlock>
    ));

    expect(document.body.textContent).toContain('Settings');
    expect(document.querySelector('span')!.textContent).toBe('child');
  });

  it('merges a custom class with the base classes', () => {
    mount(() => (
      <TitledBlock class='custom' title='Settings'>
        content
      </TitledBlock>
    ));

    const root = document.querySelector<HTMLElement>('.custom');

    expect(root).not.toBeNull();
    expect(root!.classList.contains('flex')).toBe(true);
    expect(root!.classList.contains('flex-col')).toBe(true);
  });

  it('forwards arbitrary div attributes', () => {
    mount(() => (
      <TitledBlock data-testid='block' title='Settings'>
        content
      </TitledBlock>
    ));

    expect(document.querySelector('[data-testid="block"]')).not.toBeNull();
  });
});
