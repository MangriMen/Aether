import { For } from 'solid-js';

import { Badge } from '@/shared/ui';

import { ComponentShelf } from './ComponentShelf';

export const BadgeLab = () => {
  const variants = [
    'default',
    'secondary',
    'outline',
    'success',
    'warning',
    'error',
  ] as const;

  return (
    <ComponentShelf title='Badge'>
      <div class='gap-2 flex'>
        <For each={variants}>
          {(v) => (
            <Badge class='capitalize' variant={v}>
              {v}
            </Badge>
          )}
        </For>
        <Badge round>Round</Badge>
      </div>
    </ComponentShelf>
  );
};
