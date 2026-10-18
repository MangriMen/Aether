import type { Accessor, Component, ComponentProps } from 'solid-js';

import IconCheckCircle from '~icons/mdi/check-circle';
import IconChevronDown from '~icons/mdi/chevron-down';
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
  countMirrors,
  findFailedDescendant,
  getChildren,
  getRoots,
  splitChildren,
} from '../model/progressTree';
import { ProgressCard } from './ProgressCard';

export type ProgressDetailsProps = ComponentProps<'div'> & {
  /** All bars, the tree is built from `parentId`. */
  payloads: Accessor<ProgressEvent[]>;
};

type ProgressNodeProps = {
  payload: ProgressEvent;
  payloads: Accessor<ProgressEvent[]>;
  /** Nested bars are compact. */
  size?: 'default' | 'sm';
  /** Shown inside the "Completed" group: a check mark marks a leaf. */
  finished?: boolean;
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
    getChildren(props.payloads(), props.payload.progressBarId),
  );
  const split = createMemo(() => splitChildren(children()));
  const mirrors = createMemo(() =>
    countMirrors(props.payloads(), props.payload.progressBarId),
  );
  const failedMessage = createMemo(() => {
    const status = findFailedDescendant(
      props.payloads(),
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
          <span class={cn(toggleColumn(props.size === 'sm'), 'text-success')}>
            <Show when={props.finished}>
              <IconCheckCircle class='size-4' />
            </Show>
          </span>
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
          <CollapsibleTrigger class={toggleColumn(props.size === 'sm')}>
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
                <ProgressNode
                  payload={child}
                  payloads={props.payloads}
                  size='sm'
                />
              )}
            </For>
            <Show when={split().finished.length > 0}>
              <FinishedGroup
                finished={split().finished}
                payloads={props.payloads}
              />
            </Show>
          </div>
        </CollapsibleContent>
      </Collapsible>
    </Show>
  );
};

type FinishedGroupProps = {
  finished: ProgressEvent[];
  payloads: Accessor<ProgressEvent[]>;
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
              <ProgressNode
                payload={child}
                payloads={props.payloads}
                size='sm'
                finished
              />
            )}
          </For>
        </div>
      </CollapsibleContent>
    </Collapsible>
  );
};

export const ProgressDetails: Component<ProgressDetailsProps> = (props) => {
  const [local, others] = splitProps(props, ['payloads', 'class']);

  const roots = createMemo(() => getRoots(local.payloads()));

  return (
    <div
      class={cn(
        'gap-3 flex flex-col overflow-x-hidden overflow-y-scroll',
        local.class,
      )}
      {...others}
    >
      <For each={roots()}>
        {(payload) => (
          <ProgressNode payload={payload} payloads={local.payloads} />
        )}
      </For>
    </div>
  );
};
