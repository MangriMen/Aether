import IconMdiCheck from '~icons/mdi/check';
import { For, Show } from 'solid-js';

import { Button } from '@/shared/ui';

import { ComponentShelf } from './ComponentShelf';

export const ButtonsLab = () => {
  const variants = [
    'default',
    'secondary',
    'outline',
    'ghost',
    'success',
    'destructive',
    'link',
    'ghostWarning',
    'warning',
  ] as const;
  const sizes = ['sm', 'default', 'lg', 'icon'] as const;

  return (
    <ComponentShelf title='Button'>
      <div
        class='
          gap-8
          md:grid-cols-8
          grid grid-cols-3
        '
      >
        <For each={variants}>
          {(v) => (
            <div class='space-y-4'>
              <p class='font-mono text-xs text-muted-foreground'>{v}</p>
              <div class='gap-2 flex flex-col'>
                <For each={sizes}>
                  {(s) => (
                    <Button variant={v} size={s}>
                      <Show when={s !== 'icon'} fallback={<IconMdiCheck />}>
                        Button {s}
                      </Show>
                    </Button>
                  )}
                </For>
                <Button variant={v} disabled>
                  Disabled
                </Button>
                <Button variant={v} loading>
                  Disabled
                </Button>
              </div>
            </div>
          )}
        </For>
      </div>
    </ComponentShelf>
  );
};
