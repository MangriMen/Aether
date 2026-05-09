import type { RouteSectionProps } from '@solidjs/router';
import type { Component, ComponentProps } from 'solid-js';

import { splitProps } from 'solid-js';

import { useTranslation } from '@/shared/model';
import { TitledBlock } from '@/shared/ui';

import { InstancesPanel } from './InstancesPanel';

export type HomePageProps = ComponentProps<'div'> & RouteSectionProps;

export const HomePage: Component<HomePageProps> = (props) => {
  const [local, others] = splitProps(props, [
    'params',
    'location',
    'data',
    'children',
  ]);

  const [{ t }] = useTranslation();

  return (
    <div class='p-page flex size-full flex-col overflow-hidden' {...others}>
      <TitledBlock class='h-full' title={t('home.instances')}>
        <InstancesPanel class='overflow-y-auto' />
      </TitledBlock>
      {local.children}
    </div>
  );
};
