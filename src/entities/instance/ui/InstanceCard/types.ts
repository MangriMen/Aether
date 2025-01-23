import { PolymorphicProps } from '@kobalte/core';
import { ComponentProps } from 'solid-js';

import { Instance } from '@/entities/instance';

export type InstanceCardProps = ComponentProps<'div'> & {
  instance: Instance;
  onLaunchClick?: PolymorphicProps<'button'>['onClick'];
  onStopClick?: PolymorphicProps<'button'>['onClick'];
  isLoading?: boolean;
  isRunning?: boolean;
};
