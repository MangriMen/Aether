import type { Component, ComponentProps, JSX } from 'solid-js';
import { Show, splitProps } from 'solid-js';

import { CollapsibleSettingsPane } from './CollapsibleSettingsPane';
import { cn } from '@/shared/lib';

export type SettingsPaneProps = ComponentProps<'div'> & {
  label?: JSX.Element;
  collapsible?: boolean;
  defaultOpened?: boolean;
};

export const SettingsPane: Component<SettingsPaneProps> = (props) => {
  const [local, others] = splitProps(props, [
    'label',
    'collapsible',
    'class',
    'children',
  ]);

  return (
    <Show
      when={local.collapsible}
      fallback={
        <div
          class={cn(
            'flex flex-col gap-4 rounded-lg bg-secondary-dark px-6 py-4',
            local.class,
          )}
          {...others}
        >
          <Show when={local.label}>
            <h2 class='text-2xl font-bold'>{local.label}</h2>
          </Show>
          {local.children}
        </div>
      }
    >
      <CollapsibleSettingsPane
        class={local.class}
        label={local.label}
        children={local.children}
        {...others}
      />
    </Show>
  );
};
