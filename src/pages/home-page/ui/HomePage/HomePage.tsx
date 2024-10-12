import { Component } from 'solid-js';

import { InstancesPanel } from '@/widgets/instances-panel';

import { HomePageLayout } from '../HomePageLayout';

import { HomePageProps } from '.';

export const HomePage: Component<HomePageProps> = (props) => {
  return (
    <HomePageLayout>
      <div class='p-4' {...props}>
        <InstancesPanel />
      </div>
    </HomePageLayout>
  );
};
