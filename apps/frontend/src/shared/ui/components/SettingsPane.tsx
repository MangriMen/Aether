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
          class={cn(
            `
              gap-4 rounded-md px-0
              md:pl-6
              @container/settings-pane container flex flex-col
            `,
            local.class,
          )}
          {...others}
        >
          <Show when={local.label}>
            <h2 class='pr-6 text-2xl font-semibold'>{local.label}</h2>
          </Show>
          <div
            class={cn(
              'gap-2 pr-6 pb-6 flex grow flex-col overflow-y-auto',
              local.childrenWrapperClass,
            )}
          >
            {local.children}
          </div>
        </div>
      }
    >
      <CollapsibleSettingsPane
        class={cn('@container/settings-pane', local.class)}
        label={local.label}
        children={local.children}
        {...others}
      />
    </Show>
  );
};
