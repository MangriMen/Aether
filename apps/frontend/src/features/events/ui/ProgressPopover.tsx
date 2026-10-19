import type { Accessor, Component, ComponentProps } from 'solid-js';

import IconDeleteSweep from '~icons/mdi/delete-sweep';
import IconOpenInNew from '~icons/mdi/open-in-new';
import { createMemo, For, Show, splitProps } from 'solid-js';

import { showDialog } from '@/shared/dialogs';
import { useTranslation } from '@/shared/i18n';
import { CombinedTooltip, IconButton } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import type { ProgressEvent } from '../model/progressEvent';

import {
  clearFinishedProgress,
  useProgressIndex,
} from '../model/progressStore';
import { getFinishedRoots } from '../model/progressTree';
import { ProgressCard } from './ProgressCard';
import { ProgressDetailsDialog } from './ProgressDetailsDialog';

export type ProgressPopoverProps = ComponentProps<'div'> & {
  payloads: Accessor<ProgressEvent[]>;
};

export const ProgressPopover: Component<ProgressPopoverProps> = (props) => {
  const [local, others] = splitProps(props, ['payloads', 'class']);

  const [{ t }] = useTranslation();

  const index = useProgressIndex();

  const handleOpenDetails = () => showDialog('progress', ProgressDetailsDialog);

  const hasFinished = createMemo(() => getFinishedRoots(index()).length > 0);

  return (
    <div class={cn('gap-1 flex flex-col', local.class)} {...others}>
      <div class='gap-2 px-2 flex items-center justify-between'>
        <span class='text-lg font-medium'>{t('progress.ongoingTasks')}</span>
        <div class='gap-1 flex items-center'>
          <Show when={hasFinished()}>
            <CombinedTooltip
              label={t('events.clearFinished')}
              aria-label={t('events.clearFinished')}
              as={IconButton}
              variant='ghost'
              size='sm'
              icon={IconDeleteSweep}
              onClick={clearFinishedProgress}
            />
          </Show>
          <CombinedTooltip
            label={t('progress.seeDetails')}
            aria-label={t('progress.seeDetails')}
            as={IconButton}
            class='self-end'
            variant='ghost'
            size='sm'
            icon={IconOpenInNew}
            onClick={handleOpenDetails}
          />
        </div>
      </div>
      <div class='max-h-61 gap-3 px-2 flex flex-col overflow-y-auto'>
        <For each={local.payloads()}>
          {(payload) => <ProgressCard payload={payload} />}
        </For>
      </div>
    </div>
  );
};
