import type { Accessor, Component, ComponentProps } from 'solid-js';

import IconChevronDown from '~icons/mdi/chevron-down';
import IconDownloadOutline from '~icons/mdi/download-outline';
import { createMemo, createSignal, For, Show, splitProps } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import { type ProgressEvent } from '../model/progressEvent';
import {
  findFailedDescendant,
  type ProgressIndex,
  splitChildren,
} from '../model/progressTree';
import { ProgressCard } from './ProgressCard';

export type ProgressDetailsProps = ComponentProps<'div'> & {
  index: Accessor<ProgressIndex>;
};

type ProgressNodeProps = {
  payload: ProgressEvent;
  index: Accessor<ProgressIndex>;
  /** Nested bars are compact. */
  size?: 'default' | 'sm';
};

/** The column of the chevron; leaves keep it empty so siblings line up. */
const toggleColumn = (isSmall: boolean) =>
  cn('flex w-4 shrink-0 items-center justify-center', isSmall ? 'h-5' : 'h-6');

const Chevron: Component<{ isOpen: boolean }> = (props) => (
  <IconChevronDown
    class={cn('transition-transform', { '-rotate-90': !props.isOpen })}
  />
);

const ProgressNode: Component<ProgressNodeProps> = (props) => {
  const [{ t }] = useTranslation();

  const [isOpen, setIsOpen] = createSignal(false);

  const children = createMemo(() =>
    props.index().childrenOf(props.payload.progressBarId),
  );
  // Bars keep their place while the node works; the finished ones are folded away once it
  // stops, so nothing jumps under the reader's eyes.
  const split = createMemo(() =>
    props.payload.status.type === 'running'
      ? { active: children(), finished: [] }
      : splitChildren(children()),
  );
  const mirrors = createMemo(() =>
    props.index().mirrorCount(props.payload.progressBarId),
  );
  const failedMessage = createMemo(() => {
    const status = findFailedDescendant(
      props.index(),
      props.payload.progressBarId,
    )?.status;
    return status?.type === 'failed' ? status.message : undefined;
  });

  const card = () => (
    <div class='min-w-0 gap-1 flex flex-1 flex-col'>
      <ProgressCard payload={props.payload} size={props.size} />
      <Show when={mirrors() > 0}>
        <span class='text-sm text-muted-foreground'>
          {t('events.neededFor', { count: mirrors() })}
        </span>
      </Show>
    </div>
  );

  return (
    <Show
      when={children().length > 0}
      fallback={
        <div class='gap-0.5 flex items-start'>
          <span class={toggleColumn(props.size === 'sm')} />
          {card()}
        </div>
      }
    >
      <Collapsible
        class='gap-2 flex flex-col'
        open={isOpen()}
        onOpenChange={setIsOpen}
      >
        <div class='gap-0.5 flex items-start'>
          <CollapsibleTrigger
            class={toggleColumn(props.size === 'sm')}
            aria-label={t('events.toggleDetails')}
          >
            <Chevron isOpen={isOpen()} />
          </CollapsibleTrigger>
          {card()}
        </div>
        <Show when={!isOpen() && failedMessage()}>
          <span class='text-sm text-destructive pl-4.5 truncate'>
            {failedMessage()}
          </span>
        </Show>
        <CollapsibleContent>
          <div class='gap-2 pl-4.5 flex flex-col'>
            <For each={split().active}>
              {(child) => (
                <ProgressNode payload={child} index={props.index} size='sm' />
              )}
            </For>
            <Show when={split().finished.length > 0}>
              <FinishedGroup finished={split().finished} index={props.index} />
            </Show>
          </div>
        </CollapsibleContent>
      </Collapsible>
    </Show>
  );
};

type FinishedGroupProps = {
  finished: ProgressEvent[];
  index: Accessor<ProgressIndex>;
};

/** Done and cancelled children, collapsed so they do not crowd the live ones. */
const FinishedGroup: Component<FinishedGroupProps> = (props) => {
  const [{ t }] = useTranslation();

  const [isOpen, setIsOpen] = createSignal(false);

  return (
    <Collapsible
      class='gap-2 flex flex-col'
      open={isOpen()}
      onOpenChange={setIsOpen}
    >
      <CollapsibleTrigger class='gap-0.5 text-sm text-muted-foreground flex items-center'>
        <span class={toggleColumn(true)}>
          <Chevron isOpen={isOpen()} />
        </span>
        {t('events.completedChildren', { count: props.finished.length })}
      </CollapsibleTrigger>
      <CollapsibleContent>
        <div class='gap-2 pl-4.5 flex flex-col opacity-70'>
          <For each={props.finished}>
            {(child) => (
              <ProgressNode payload={child} index={props.index} size='sm' />
            )}
          </For>
        </div>
      </CollapsibleContent>
    </Collapsible>
  );
};

export const ProgressDetails: Component<ProgressDetailsProps> = (props) => {
  const [local, others] = splitProps(props, ['index', 'class']);

  const [{ t }] = useTranslation();

  return (
    <div
      class={cn(
        'gap-3 flex flex-col overflow-x-hidden overflow-y-auto',
        local.class,
      )}
      {...others}
    >
      <For
        each={local.index().roots}
        fallback={
          <div class='gap-2 py-10 text-muted-foreground flex flex-col items-center text-center'>
            <IconDownloadOutline class='size-10 opacity-50' />
            <span class='font-medium'>{t('events.nothingRunning')}</span>
            <span class='text-sm'>{t('events.nothingRunningHint')}</span>
          </div>
        }
      >
        {(payload) => <ProgressNode payload={payload} index={local.index} />}
      </For>
    </div>
  );
};
