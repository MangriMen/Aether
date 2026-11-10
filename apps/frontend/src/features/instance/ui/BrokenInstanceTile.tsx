import type { PolymorphicProps } from '@kobalte/core';
import type { ValidComponent } from 'solid-js';

import IconMdiAlertCircleOutline from '~icons/mdi/alert-circle-outline';
import { mergeProps, splitProps } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { CombinedTooltip } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import {
  getBrokenReasonText,
  type BrokenInstance,
} from '../model/brokenInstance';
import { INSTANCE_CARD_CLASS } from '../model/instanceCardClass';
import { InstanceIcon } from './InstanceIcon';

export type BrokenInstanceTileProps = {
  instance: BrokenInstance;
  class?: string;
};

/**
 * The card of an instance the launcher cannot read: the size and the look of an instance's card,
 * with a mark where the state of a running one goes. The reason is in the tooltip, and in full
 * in the details the click opens.
 */
export const BrokenInstanceTile = <T extends ValidComponent = 'button'>(
  props: PolymorphicProps<T, BrokenInstanceTileProps>,
) => {
  const merged = mergeProps({ as: 'button' }, props);

  const [local, others] = splitProps(merged, ['instance', 'class']);

  const [{ t }] = useTranslation();

  return (
    <CombinedTooltip
      class={cn(local.class, INSTANCE_CARD_CLASS)}
      placement='bottom'
      label={
        <div class='max-w-64 gap-0.5 flex flex-col'>
          <span class='font-medium break-all'>{local.instance.id}</span>
          <span class='wrap-break-word line-clamp-4'>
            {getBrokenReasonText(local.instance.reason, t)}
          </span>
        </div>
      }
      {...others}
    >
      <div class='flex items-center justify-center'>
        <InstanceIcon
          class='size-28 opacity-50 grayscale'
          alt={t('instance.broken.title')}
        />
      </div>
      <div class='flex flex-col select-none'>
        <div class='truncate'>{local.instance.id}</div>
        <div class='text-sm text-muted-foreground'>
          {t('instance.broken.label')}
        </div>
      </div>
      <IconMdiAlertCircleOutline
        class='
          top-2 right-2 size-5 rounded-full bg-card text-destructive absolute
        '
        aria-hidden='true'
      />
    </CombinedTooltip>
  );
};
