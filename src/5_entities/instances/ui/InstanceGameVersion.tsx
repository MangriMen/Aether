import type { ModLoader } from '@/5_entities/minecraft/@x/instances';
import { cn } from '@/shared/lib';
import { useTranslate } from '@/shared/model';
import { CombinedTooltip } from '@/shared/ui';
import { Icon } from '@iconify-icon/solid';
import { splitProps, type Component, type ComponentProps } from 'solid-js';
import MdiGamepadSquare from '@iconify/icons-mdi/gamepad-square';

export type InstanceGameVersionProps = ComponentProps<'div'> & {
  loader: ModLoader;
  gameVersion: string;
};

export const InstanceGameVersion: Component<InstanceGameVersionProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['loader', 'gameVersion', 'class']);

  const [{ t }] = useTranslate();

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
