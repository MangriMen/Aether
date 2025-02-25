import type { Component, ComponentProps, JSX } from 'solid-js';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

export type SettingsPaneProps = ComponentProps<'div'> & {
  label?: JSX.Element;
};

export const SettingsPane: Component<SettingsPaneProps> = (props) => {
  const [local, others] = splitProps(props, ['label', 'class', 'children']);

  return (
    <div
      class={cn(
        'flex flex-col gap-4 rounded-lg bg-secondary-dark px-6 py-4',
        local.class,
      )}
      {...others}
    >
      <h2 class='text-xl font-bold'>{local.label}</h2>
      {local.children}
    </div>
  );
};
