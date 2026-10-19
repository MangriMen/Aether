import type { JSX } from 'solid-js';

import { createSignal } from 'solid-js';
import { render } from 'solid-js/web';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { PaginationItem } from '../../primitives/Pagination';
import { CombinedPagination } from '../CombinedPagination';

let dispose: (() => void) | undefined;

const mount = (ui: () => JSX.Element) => {
  dispose = render(ui, document.body);
};

const buttons = () => [...document.querySelectorAll('button')];

const pageButtons = () =>
  buttons().filter((button) => /^\d+$/.test(button.textContent ?? ''));

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('CombinedPagination', () => {
  it('renders a page button per page', () => {
    mount(() => <CombinedPagination count={3} page={1} />);

    expect(pageButtons().map((button) => button.textContent)).toEqual([
      '1',
      '2',
      '3',
    ]);
  });

  it('renders previous and next buttons', () => {
    mount(() => <CombinedPagination count={3} page={2} />);

    expect(document.body.textContent).toContain('Previous');
    expect(document.body.textContent).toContain('Next');
  });

  it('calls onPageChange when another page is clicked', () => {
    const onPageChange = vi.fn();

    mount(() => (
      <CombinedPagination count={3} onPageChange={onPageChange} page={1} />
    ));

    pageButtons()[1]!.click();

    expect(onPageChange).toHaveBeenCalledWith(2);
  });

  it('calls onPageChange when next is clicked', () => {
    const onPageChange = vi.fn();

    mount(() => (
      <CombinedPagination count={3} onPageChange={onPageChange} page={1} />
    ));

    buttons()
      .find((button) => button.textContent?.includes('Next'))!
      .click();

    expect(onPageChange).toHaveBeenCalledWith(2);
  });

  it('follows a controlled page signal', () => {
    const [page, setPage] = createSignal(1);

    mount(() => <CombinedPagination count={3} page={page()} />);

    expect(pageButtons()[0]!.getAttribute('data-current')).not.toBeNull();

    setPage(2);

    expect(pageButtons()[1]!.getAttribute('data-current')).not.toBeNull();
  });

  it('uses a custom itemComponent', () => {
    mount(() => (
      <CombinedPagination
        count={2}
        itemComponent={(props) => (
          <PaginationItem page={props.page}>p{props.page}</PaginationItem>
        )}
        page={1}
      />
    ));

    expect(document.body.textContent).toContain('p1');
    expect(document.body.textContent).toContain('p2');
  });

  it('merges a custom class with the base classes', () => {
    mount(() => <CombinedPagination class='custom' count={2} page={1} />);

    const root = document.querySelector<HTMLElement>('.custom');

    expect(root).not.toBeNull();
    expect(root!.classList.contains('custom')).toBe(true);
    expect(root!.tagName).toBe('NAV');
  });
});
