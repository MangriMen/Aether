import { Icon } from '@iconify-icon/solid';
import MdiGamepadSquare from '@iconify/icons-mdi/gamepad-square';
import { type Component, type ComponentProps, splitProps } from 'solid-js';

import type { ModLoader } from '@/entities/minecraft/@x/instances';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTooltip } from '@/shared/ui';

export type InstanceGameVersionProps = {
  gameVersion: string;
  loader: ModLoader;
} & ComponentProps<'div'>;

export const InstanceGameVersion: Component<InstanceGameVersionProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['loader', 'gameVersion', 'class']);

  const [{ t }] = useTranslation();

  return (
    <CombinedTooltip
      as='span'
      class={cn('capitalize inline-flex items-center gap-1', local.class)}
      label={t('common.gameVersion')}
      {...others}
    >
      <Icon icon={MdiGamepadSquare} />
      {local.loader} {local.gameVersion}
    </CombinedTooltip>
  );
};
