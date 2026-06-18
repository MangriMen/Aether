import { splitProps, type Component } from 'solid-js';

import { cn } from '@/shared/lib';
import { Image, type ImageProps } from '@/shared/ui';

export type InstanceIconProps = ImageProps;

export const InstanceIcon: Component<InstanceIconProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  return (
    <Image
      class={cn(
        'aspect-square min-w-max [&_img]:object-contain overflow-hidden',
        local.class,
      )}
      {...others}
    />
  );
};
