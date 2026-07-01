import type { Component, ComponentProps, JSX } from 'solid-js';

import { Show, splitProps } from 'solid-js';

import { FieldLabel } from '..';
import { cn } from '../../lib';

export type SettingsEntryProps = ComponentProps<'div'> & {
  title?: string | JSX.Element;
  description?: string | JSX.Element;
  variant?: 'card';
};

export const SettingsEntry: Component<SettingsEntryProps> = (props) => {
  const [local, others] = splitProps(props, [
    'title',
    'description',
    'variant',
    'class',
    'children',
  ]);

  return (
    <div
      class={cn(
        'flex w-full flex-col justify-between gap-8 @sm/settings-pane:flex-row @sm/settings-pane:items-center',
        local.class,
        {
          'bg-card/card border p-4 rounded-md': local.variant === 'card',
        },
      )}
      {...others}
    >
      <div class='text-pretty'>
        <Show when={local.title}>
          <FieldLabel class='text-lg font-medium'>{local.title}</FieldLabel>
        </Show>
        <Show when={local.description}>
          <p class='text-sm whitespace-pre-line text-muted-foreground'>
            {local.description}
          </p>
        </Show>
      </div>
      {local.children}
    </div>
  );
};
