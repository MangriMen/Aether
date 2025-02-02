import type { Component, ComponentProps } from 'solid-js';
import { splitProps } from 'solid-js';

import { TitledBlock } from '@/shared/ui';

import { useTranslate } from '@/shared/model';
import type { RouteSectionProps } from '@solidjs/router';
import { InstancesPanel } from './InstancesPanel';

export type HomePageProps = ComponentProps<'div'> & RouteSectionProps;

export const HomePage: Component<HomePageProps> = (props) => {
  const [local, others] = splitProps(props, [
    'params',
    'location',
    'data',
    'children',
  ]);

  const [{ t }] = useTranslate();

  return (
    <div class='size-full p-4' {...others}>
      <TitledBlock title={t('home.instances')}>
        <InstancesPanel />
      </TitledBlock>
      {local.children}
    </div>
  );
};
