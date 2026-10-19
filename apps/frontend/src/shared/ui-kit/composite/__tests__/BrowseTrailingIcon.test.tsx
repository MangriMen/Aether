import type { JSX } from 'solid-js';

import { render } from 'solid-js/web';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { UiKitLabelsProvider } from '../../primitives/UiKitLabels';
import { BrowseTrailingIcon } from '../BrowseTrailingIcon';

let dispose: (() => void) | undefined;

const mount = (ui: () => JSX.Element) => {
  dispose = render(ui, document.body);
};

const button = () => document.querySelector<HTMLButtonElement>('button')!;

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('BrowseTrailingIcon', () => {
  it('renders a button with an icon', () => {
    mount(() => <BrowseTrailingIcon onBrowse={() => {}} />);

    expect(button()).not.toBeNull();
    expect(button().querySelector('svg')).not.toBeNull();
  });

  it('calls onBrowse when clicked', () => {
    const onBrowse = vi.fn();

    mount(() => <BrowseTrailingIcon onBrowse={onBrowse} />);

    button().click();

    expect(onBrowse).toHaveBeenCalledTimes(1);
  });

  it('renders inside a labels provider', () => {
    mount(() => (
      <UiKitLabelsProvider labels={{ browse: 'Обзор' }}>
        <BrowseTrailingIcon onBrowse={() => {}} />
      </UiKitLabelsProvider>
    ));

    expect(button()).not.toBeNull();
  });

  it('renders a square icon button', () => {
    mount(() => <BrowseTrailingIcon onBrowse={() => {}} />);

    expect(button().classList.contains('aspect-square')).toBe(true);
    expect(button().classList.contains('size-7')).toBe(true);
  });
});
