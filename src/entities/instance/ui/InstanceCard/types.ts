import { PolymorphicProps } from '@kobalte/core';
import { ComponentProps } from 'solid-js';

// eslint-disable-next-line boundaries/element-types
import { Instance } from '@/entities/minecraft';

export type InstanceCardProps = ComponentProps<'div'> & {
  instance: Instance;
  onLaunchClick?: PolymorphicProps<'button'>['onClick'];
  onStopClick?: PolymorphicProps<'button'>['onClick'];
  isLoading?: boolean;
  isRunning?: boolean;
};
