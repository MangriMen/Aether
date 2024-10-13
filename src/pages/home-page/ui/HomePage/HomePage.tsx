import { Component } from 'solid-js';

import { InstancesPanel } from '@/widgets/instances-panel';

import { HomePageProps } from '.';

export const HomePage: Component<HomePageProps> = (props) => {
  return (
    <div class='size-full p-4' {...props}>
      <h2 class='text-xl font-bold'>Instances</h2>
      <InstancesPanel />
    </div>
  );
};
