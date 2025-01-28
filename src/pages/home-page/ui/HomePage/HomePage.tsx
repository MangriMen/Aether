import type { Component } from 'solid-js';
import { splitProps } from 'solid-js';

import { TitledBlock } from '@/shared/ui';

import { InstancesPanel } from '@/widgets/instances-panel';


import { useTranslate } from '@/app/model';

import type { HomePageProps } from '.';

export const HomePage: Component<HomePageProps> = (props) => {
  const [_, others] = splitProps(props, ['params', 'location', 'data']);

  const [{ t }] = useTranslate();

  return (
    <div class='size-full p-4' {...others}>
      <TitledBlock title={t('home.instances')}>
        <InstancesPanel />
      </TitledBlock>
    </div>
  );
};
