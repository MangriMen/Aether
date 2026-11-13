import { render } from 'solid-js/web';
import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '../DropdownMenu';

let dispose: (() => void) | undefined;

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('DropdownMenuItem', () => {
  it('should_call_onSelect_when_enter_is_pressed_on_a_menu_item', () => {
    const onSelect = vi.fn();

    dispose = render(
      () => (
        <DropdownMenu open>
          <DropdownMenuTrigger>Open</DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuItem onSelect={onSelect}>Item</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      ),
      document.body,
    );

    const item = document.querySelector<HTMLElement>('[role="menuitem"]');
    expect(item).not.toBeNull();

    item?.focus();
    item?.dispatchEvent(
      new KeyboardEvent('keydown', {
        key: 'Enter',
        bubbles: true,
        cancelable: true,
      }),
    );

    expect(onSelect).toHaveBeenCalledTimes(1);
  });
});
