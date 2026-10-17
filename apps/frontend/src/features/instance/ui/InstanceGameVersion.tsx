import IconMdiGamepadSquare from '~icons/mdi/gamepad-square';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import type { ModLoader } from '@/features/minecraft';

import { useTranslation } from '@/shared/i18n';
import { CombinedTooltip } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

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
        'gap-1 inline-flex max-w-max items-center capitalize',
        local.class,
      )}
      {...others}
    >
      <IconMdiGamepadSquare />
      {local.loader} {local.gameVersion}
    </CombinedTooltip>
  );
};
