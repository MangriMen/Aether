import { onCleanup } from 'solid-js';

/** Put on what the arrow keys move between; anything else inside the container is skipped. */
export const ARROW_NAV_ITEM_ATTRIBUTE = 'data-arrow-nav-item';

const ARROW_KEYS = ['ArrowRight', 'ArrowLeft', 'ArrowDown', 'ArrowUp'];

/** The item one row above or below `from`, in the column nearest to it. */
const itemInNextRow = (
  items: HTMLElement[],
  from: HTMLElement,
  direction: 1 | -1,
): HTMLElement | undefined => {
  const origin = from.getBoundingClientRect();
  const originCenter = origin.left + origin.width / 2;

  const rows = items
    .map((item) => ({ item, rect: item.getBoundingClientRect() }))
    // Items of the same row share a top, give or take a pixel of rounding.
    .filter(({ rect }) =>
      direction === 1 ? rect.top > origin.top + 1 : rect.top < origin.top - 1,
    );

  if (rows.length === 0) {
    return undefined;
  }

  // The nearest row in that direction.
  const nearestTop = rows.reduce(
    (best, { rect }) =>
      direction === 1 ? Math.min(best, rect.top) : Math.max(best, rect.top),
    direction === 1 ? Infinity : -Infinity,
  );

  return rows
    .filter(({ rect }) => Math.abs(rect.top - nearestTop) <= 1)
    .reduce((best, candidate) => {
      const distance = (r: DOMRect) =>
        Math.abs(r.left + r.width / 2 - originCenter);
      return distance(candidate.rect) < distance(best.rect) ? candidate : best;
    }).item;
};

/**
 * Arrow keys move focus between the items marked with {@link ARROW_NAV_ITEM_ATTRIBUTE}:
 * left and right to the neighbour (wrapping), up and down to the same column of the row above or
 * below in a wrapped grid.
 */
export const useArrowNavigation = (
  selector = `[${ARROW_NAV_ITEM_ATTRIBUTE}]`,
) => {
  let containerRef: HTMLDivElement | undefined;

  const onKeyDown = (e: KeyboardEvent) => {
    if (!ARROW_KEYS.includes(e.key) || !containerRef) {
      return;
    }

    const elements = Array.from(
      containerRef.querySelectorAll<HTMLElement>(selector),
    );

    const current = elements.find(
      (element) => element === document.activeElement,
    );

    if (!current) {
      return;
    }

    e.preventDefault();

    const index = elements.indexOf(current);
    let next: HTMLElement | undefined;

    switch (e.key) {
      case 'ArrowRight':
        next = elements[(index + 1) % elements.length];
        break;
      case 'ArrowLeft':
        next = elements[(index - 1 + elements.length) % elements.length];
        break;
      case 'ArrowDown':
        next = itemInNextRow(elements, current, 1);
        break;
      case 'ArrowUp':
        next = itemInNextRow(elements, current, -1);
        break;
    }

    next?.focus();
  };

  onCleanup(() => {
    containerRef = undefined;
  });

  return {
    // Solid passes undefined to ref callbacks on unmount.
    setRef: (el: HTMLDivElement | undefined) => {
      containerRef = el;
    },
    onKeyDown,
  };
};
