import type { Component, ComponentProps, JSX } from 'solid-js';

import { Show, splitProps } from 'solid-js';

import { cn } from '../../lib';
import { CollapsibleSettingsPane } from './CollapsibleSettingsPane';

export type SettingsPaneProps = ComponentProps<'div'> & {
  label?: JSX.Element;
  collapsible?: boolean;
  defaultOpened?: boolean;
  childrenWrapperClass?: string;
};

export const SettingsPane: Component<SettingsPaneProps> = (props) => {
  const [local, others] = splitProps(props, [
    'label',
    'collapsible',
    'childrenWrapperClass',
    'class',
    'children',
  ]);

  return (
    <Show
      when={local.collapsible}
      fallback={
        <div
          class={cn('flex flex-col gap-4 rounded-lg', local.class)}
          {...others}
        >
          <Show when={local.label}>
            <h2 class='text-2xl font-semibold'>{local.label}</h2>
          </Show>
          <div
            class={cn(
              'flex flex-col gap-2 overflow-y-auto grow',
              local.childrenWrapperClass,
            )}
          >
            {local.children}
          </div>
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
