import { Component, splitProps } from 'solid-js';

import { SettingsPaneProps } from './types';

export const SettingsPane: Component<SettingsPaneProps> = (props) => {
  const [local, others] = splitProps(props, ['title', 'class', 'children']);

  return (
    <div
      class='flex flex-col gap-2 rounded-lg bg-secondary-dark px-6 py-4'
      {...others}
    >
      <h2 class='text-xl font-bold'>{local.title}</h2>
      {local.children}
    </div>
  );
};
