import { splitProps, type Component } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { Image, type ImageProps } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

export type InstanceIconProps = ImageProps;

export const InstanceIcon: Component<InstanceIconProps> = (props) => {
  const [local, others] = splitProps(props, ['class', 'alt']);
  const [{ t }] = useTranslation();

  return (
    <Image
      alt={local.alt ?? t('instance.icon')}
      class={cn('aspect-square min-w-max overflow-hidden', local.class)}
      imgClass='object-contain'
      {...others}
    />
  );
};
