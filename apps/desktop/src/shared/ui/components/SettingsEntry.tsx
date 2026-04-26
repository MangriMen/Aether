import type { Component, ComponentProps, JSX } from 'solid-js';

import { Show, splitProps } from 'solid-js';

import { FieldLabel } from '..';
import { cn } from '../../lib';

export type SettingsEntryProps = ComponentProps<'div'> & {
  title?: string | JSX.Element;
  description?: string | JSX.Element;
  isTopLevel?: boolean;
};

export const SettingsEntry: Component<SettingsEntryProps> = (props) => {
  const [local, others] = splitProps(props, [
    'title',
    'description',
    'isTopLevel',
    'class',
    'children',
  ]);

  return (
    <div
      class={cn('flex justify-between items-center w-full gap-8', local.class)}
      {...others}
    >
      <div class='text-pretty'>
        <Show when={local.title}>
          <FieldLabel
            class={cn('text-lg font-semibold', {
              'font-bold': local.isTopLevel,
            })}
          >
            {local.title}
          </FieldLabel>
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
