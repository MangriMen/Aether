import { Component } from 'solid-js';

import { TitledBlock } from '@/shared/ui';

import { InstancesPanel } from '@/widgets/instances-panel';

import { HomePageProps } from '.';

export const HomePage: Component<HomePageProps> = (props) => {
  return (
    <div class='size-full p-4' {...props}>
      <TitledBlock title='Instances'>
        <InstancesPanel />
      </TitledBlock>
    </div>
  );
};
