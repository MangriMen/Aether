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
  ] as const;
  const sizes = ['sm', 'default', 'lg', 'icon'] as const;

  return (
    <ComponentShelf title='Button'>
      <div class='grid grid-cols-3 gap-8 md:grid-cols-8'>
        <For each={variants}>
          {(v) => (
            <div class='space-y-4'>
              <p class='font-mono text-xs text-muted-foreground'>{v}</p>
              <div class='flex flex-col gap-2'>
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
              </div>
            </div>
          )}
        </For>
      </div>
    </ComponentShelf>
  );
};
