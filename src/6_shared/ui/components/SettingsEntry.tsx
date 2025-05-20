import type { Component, ComponentProps, JSX } from 'solid-js';
import { Show, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { FieldLabel } from '@/shared/ui';

export type SettingsEntryProps = ComponentProps<'div'> & {
  title?: string | JSX.Element;
  description?: string | JSX.Element;
};

export const SettingsEntry: Component<SettingsEntryProps> = (props) => {
  const [local, others] = splitProps(props, [
    'title',
    'description',
    'class',
    'children',
  ]);

  return (
    <div
      class={cn('flex justify-between items-center w-full gap-8', local.class)}
      {...others}
    >
      <div class=' text-pretty'>
        <Show when={local.title}>
          <FieldLabel class='text-lg font-bold'>{local.title}</FieldLabel>
        </Show>
        <Show when={local.description}>
          <p class='whitespace-pre-line text-muted-foreground'>
            {local.description}
          </p>
        </Show>
      </div>
      {local.children}
    </div>
  );
};
