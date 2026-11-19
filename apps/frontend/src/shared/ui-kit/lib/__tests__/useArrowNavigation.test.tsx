import { render } from 'solid-js/web';
import { afterEach, describe, expect, it } from 'vitest';

import { useArrowNavigation } from '../useArrowNavigation';

let dispose: (() => void) | undefined;

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

/** A grid of three columns: the cells are laid out by hand, as jsdom has no layout. */
const mountGrid = (cells: number) => {
  const { setRef, onKeyDown } = useArrowNavigation();

  dispose = render(
    () => (
      <div role='presentation' ref={setRef} onKeyDown={onKeyDown}>
        {Array.from({ length: cells }, (_, i) => (
          <button
            data-arrow-nav-item=''
            data-testid={`cell-${i}`}
            ref={(el) => {
              el.getBoundingClientRect = () =>
                new DOMRect((i % 3) * 100, Math.floor(i / 3) * 100, 90, 90);
            }}
          />
        ))}
        <button data-testid='extra' />
      </div>
    ),
    document.body,
  );

  return (i: number) =>
    document.querySelector<HTMLElement>(`[data-testid="cell-${i}"]`)!;
};

const press = (from: HTMLElement, key: string) => {
  from.focus();
  from.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true }));
};

describe('useArrowNavigation', () => {
  it('moves down to the same column of the next row and back up', () => {
    const cell = mountGrid(8);

    press(cell(1), 'ArrowDown');
    expect(document.activeElement).toBe(cell(4));

    press(cell(4), 'ArrowUp');
    expect(document.activeElement).toBe(cell(1));
  });

  it('takes the nearest column when the next row is short', () => {
    const cell = mountGrid(8);

    press(cell(5), 'ArrowUp');
    expect(document.activeElement).toBe(cell(2));

    // Row 2 has only cells 6 and 7: from column 2 the nearest is 7.
    press(cell(5), 'ArrowDown');
    expect(document.activeElement).toBe(cell(7));
  });

  it('stays put at the first and last row', () => {
    const cell = mountGrid(6);

    press(cell(0), 'ArrowUp');
    expect(document.activeElement).toBe(cell(0));

    press(cell(5), 'ArrowDown');
    expect(document.activeElement).toBe(cell(5));
  });

  it('wraps left and right, and skips what is not marked', () => {
    const cell = mountGrid(4);

    press(cell(3), 'ArrowRight');
    expect(document.activeElement).toBe(cell(0));

    press(cell(0), 'ArrowLeft');
    expect(document.activeElement).toBe(cell(3));
  });

  it('ignores a key pressed on something that is not an item', () => {
    mountGrid(3);
    const extra = document.querySelector<HTMLElement>('[data-testid="extra"]')!;

    press(extra, 'ArrowRight');
    expect(document.activeElement).toBe(extra);
  });
});
