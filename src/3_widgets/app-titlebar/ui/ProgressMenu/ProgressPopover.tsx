import type { Accessor, Component, ComponentProps } from 'solid-js';

import OpenInNewIcon from '@iconify/icons-mdi/open-in-new';
import { For, splitProps } from 'solid-js';

import type { LoadingPayload } from '@/entities/events';

import { ProgressCard } from '@/entities/events';
import { cn } from '@/shared/lib';
import { closeDialog, showDialog, useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton } from '@/shared/ui';

import { ProgressDetailsDialog } from './ProgressDetailsDialog';

export type ProgressPopoverProps = ComponentProps<'div'> & {
  payloads: Accessor<LoadingPayload[]>;
};

export const ProgressPopover: Component<ProgressPopoverProps> = (props) => {
  const [local, others] = splitProps(props, ['payloads', 'class']);

  const [{ t }] = useTranslation();

  const handleOpenDetails = () =>
    showDialog('progress', ProgressDetailsDialog, {
      defaultOpen: true,
      onOpenChange: () => closeDialog('progress'),
    });

  return (
    <div class={cn('flex flex-col gap-1', local.class)} {...others}>
      <div class='flex items-center justify-between gap-2'>
        <span class='text-lg font-medium'>{t('progress.ongoingTasks')}</span>
        <CombinedTooltip
          label={t('progress.seeDetails')}
          as={IconButton}
          class='self-end'
          variant='ghost'
          size='sm'
          icon={OpenInNewIcon}
          onClick={handleOpenDetails}
        />
      </div>
      <div class='flex max-h-[244px] flex-col gap-3 overflow-hidden'>
        <For each={local.payloads()}>
          {(payload) => <ProgressCard payload={payload} />}
        </For>
      </div>
    </div>
  );
};
