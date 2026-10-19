import { render } from 'solid-js/web';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import { Dialog, DialogContent } from '../Dialog';

const KEY = 'test:dialog-width';

let dispose: (() => void) | undefined;

const mount = () => {
  dispose = render(
    () => (
      <Dialog open>
        <DialogContent
          resizable
          resizeStorageKey={KEY}
          defaultWidth={500}
          minWidth={300}
        >
          body
        </DialogContent>
      </Dialog>
    ),
    document.body,
  );
};

const edges = () =>
  [...document.querySelectorAll<HTMLElement>('[role="separator"]')] as [
    HTMLElement,
    HTMLElement,
  ];

const content = () => document.querySelector<HTMLElement>('[role="dialog"]')!;

const drag = (edge: HTMLElement, from: number, to: number) => {
  const init = { bubbles: true, pointerId: 1 };
  edge.dispatchEvent(new MouseEvent('pointerdown', { ...init, clientX: from }));
  edge.dispatchEvent(new MouseEvent('pointermove', { ...init, clientX: to }));
  edge.dispatchEvent(new MouseEvent('pointerup', { ...init, clientX: to }));
};

beforeEach(() => {
  localStorage.clear();
  HTMLElement.prototype.setPointerCapture = () => {};
});

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('resizable DialogContent', () => {
  it('starts at the default width', () => {
    mount();

    expect(content().style.width).toBe('500px');
  });

  it('grows on both sides when an edge is dragged outwards', () => {
    mount();
    const [left, right] = edges();

    drag(right, 100, 150);
    expect(content().style.width).toBe('600px');

    drag(left, 100, 50);
    expect(content().style.width).toBe('700px');
  });

  it('does not shrink below the minimum', () => {
    mount();

    drag(edges()[1], 100, -500);

    expect(content().style.width).toBe('300px');
  });

  it('keeps the width and resets it on double click', () => {
    mount();
    const [, right] = edges();

    drag(right, 100, 150);
    expect(localStorage.getItem(KEY)).toBe('600');

    right.dispatchEvent(new MouseEvent('dblclick', { bubbles: true }));
    expect(content().style.width).toBe('500px');
    expect(localStorage.getItem(KEY)).toBe('500');
  });

  it('has no edges when it is not resizable', () => {
    dispose = render(
      () => (
        <Dialog open>
          <DialogContent>body</DialogContent>
        </Dialog>
      ),
      document.body,
    );

    expect(edges()).toHaveLength(0);
  });
});
