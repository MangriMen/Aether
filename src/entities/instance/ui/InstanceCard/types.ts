import type { ComponentProps } from 'solid-js';

import type { Instance } from '@/entities/instance';

export type InstanceCardProps = ComponentProps<'div'> & {
  instance: Instance;
  onLaunchClick?: ComponentProps<'button'>['onClick'];
  onStopClick?: ComponentProps<'button'>['onClick'];
  isLoading?: boolean;
  isRunning?: boolean;
};
