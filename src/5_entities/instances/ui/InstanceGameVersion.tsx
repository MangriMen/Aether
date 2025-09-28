import { Icon } from '@iconify-icon/solid';
import MdiGamepadSquare from '@iconify/icons-mdi/gamepad-square';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import type { ModLoader } from '@/entities/minecraft/@x/instances';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTooltip } from '@/shared/ui';

export type InstanceGameVersionProps = ComponentProps<'div'> & {
  loader: ModLoader;
  gameVersion: string;
};

export const InstanceGameVersion: Component<InstanceGameVersionProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['loader', 'gameVersion', 'class']);

  const [{ t }] = useTranslation();

  return (
    <CombinedTooltip
      label={t('common.gameVersion')}
      as='span'
      class={cn('capitalize inline-flex items-center gap-1', local.class)}
      {...others}
    >
      <Icon icon={MdiGamepadSquare} />
      {local.loader} {local.gameVersion}
    </CombinedTooltip>
  );
};
