import IconMdiGamepadSquare from '~icons/mdi/gamepad-square';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTooltip } from '@/shared/ui';

import type { ModLoader } from '../model';

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
      class={cn(
        'capitalize inline-flex max-w-max items-center gap-1',
        local.class,
      )}
      {...others}
    >
      <IconMdiGamepadSquare />
      {local.loader} {local.gameVersion}
    </CombinedTooltip>
  );
};
