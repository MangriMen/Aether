import { ComponentProps } from 'solid-js';

// eslint-disable-next-line boundaries/element-types
import { Instance } from '@/entities/minecraft';

export type InstanceCardProps = ComponentProps<'div'> & {
  instance: Instance;
  onLaunchClick?: () => void;
  isLoading?: boolean;
};
