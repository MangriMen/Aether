import type { Component } from 'solid-js';
import { splitProps } from 'solid-js';

import { TitledBlock } from '@/shared/ui';

import { InstancesPanel } from '@/widgets/instances-panel';

import type { HomePageProps } from '.';

export const HomePage: Component<HomePageProps> = (props) => {
  const [_, others] = splitProps(props, ['params', 'location', 'data']);
  return (
    <div class='size-full p-4' {...others}>
      <TitledBlock title='Instances'>
        <InstancesPanel />
      </TitledBlock>
    </div>
  );
};
