import type { Component, ComponentProps, JSX } from 'solid-js';

import { Show, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import { CollapsibleSettingsPane } from './CollapsibleSettingsPane';

export type SettingsPaneProps = {
  collapsible?: boolean;
  defaultOpened?: boolean;
  label?: JSX.Element;
} & ComponentProps<'div'>;

export const SettingsPane: Component<SettingsPaneProps> = (props) => {
  const [local, others] = splitProps(props, [
    'label',
    'collapsible',
    'class',
    'children',
  ]);

  return (
    <Show
      fallback={
        <div
          class={cn('flex flex-col gap-4 rounded-lg px-6 py-4', local.class)}
          {...others}
        >
          <Show when={local.label}>
            <h2 class='text-2xl font-bold'>{local.label}</h2>
          </Show>
          {local.children}
        </div>
      }
      when={local.collapsible}
    >
      <CollapsibleSettingsPane
        children={local.children}
        class={local.class}
        label={local.label}
        {...others}
      />
    </Show>
  );
};
