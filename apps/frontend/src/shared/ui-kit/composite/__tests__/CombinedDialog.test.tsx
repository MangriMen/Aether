import type { JSX } from 'solid-js';

import { render } from 'solid-js/web';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { UiKitLabelsProvider } from '../../primitives/UiKitLabels';
import { CombinedDialog } from '../CombinedDialog';

let dispose: (() => void) | undefined;

const mount = (ui: () => JSX.Element) => {
  dispose = render(ui, document.body);
};

const buttons = () => [...document.querySelectorAll('button')];

const buttonByText = (text: string) =>
  buttons().find((button) => button.textContent === text)!;

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('CombinedDialog', () => {
  it('renders nothing while closed', () => {
    mount(() => <CombinedDialog header='Title' open={false} />);

    expect(document.querySelector('[role="dialog"]')).toBeNull();
  });

  it('renders the header and description when open', () => {
    mount(() => (
      <CombinedDialog description='Some description' header='Title' open />
    ));

    expect(document.querySelector('[role="dialog"]')).not.toBeNull();
    expect(document.body.textContent).toContain('Title');
    expect(document.body.textContent).toContain('Some description');
  });

  it('renders the default ok and cancel buttons', () => {
    mount(() => <CombinedDialog open />);

    expect(buttonByText('Ok')).toBeDefined();
    expect(buttonByText('Cancel')).toBeDefined();
  });

  it('uses custom button texts', () => {
    mount(() => (
      <CombinedDialog buttonCancelText='Nope' buttonOkText='Yep' open />
    ));

    expect(buttonByText('Yep')).toBeDefined();
    expect(buttonByText('Nope')).toBeDefined();
  });

  it('uses labels from the provider', () => {
    mount(() => (
      <UiKitLabelsProvider labels={{ cancel: 'Отмена', ok: 'Ок' }}>
        <CombinedDialog open />
      </UiKitLabelsProvider>
    ));

    expect(buttonByText('Ок')).toBeDefined();
    expect(buttonByText('Отмена')).toBeDefined();
  });

  it('calls onOk and onCancel', () => {
    const onOk = vi.fn();
    const onCancel = vi.fn();

    mount(() => <CombinedDialog onCancel={onCancel} onOk={onOk} open />);

    buttonByText('Ok').click();
    buttonByText('Cancel').click();

    expect(onOk).toHaveBeenCalledTimes(1);
    expect(onCancel).toHaveBeenCalledTimes(1);
  });

  it('renders no header or description when they are omitted', () => {
    mount(() => <CombinedDialog open />);

    expect(document.body.textContent).not.toContain('Title');
    expect(document.querySelector('[role="dialog"]')).not.toBeNull();
  });
});
