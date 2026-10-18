import type { Accessor, Component, ComponentProps } from 'solid-js';

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
} from '../model/progressTree';
import { ProgressCard } from './ProgressCard';

export type ProgressDetailsProps = ComponentProps<'div'> & {
  /** All bars, the tree is built from `parentId`. */
  payloads: Accessor<ProgressEvent[]>;
};

type ProgressNodeProps = {
  payload: ProgressEvent;
  payloads: Accessor<ProgressEvent[]>;
};

const ProgressNode: Component<ProgressNodeProps> = (props) => {
  const [{ t }] = useTranslation();

  const [isOpen, setIsOpen] = createSignal(false);

  const children = createMemo(() =>
    getChildren(props.payloads(), props.payload.progressBarId),
  );
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
    <div class='gap-1 flex flex-col'>
      <ProgressCard payload={props.payload} />
      <Show when={mirrors() > 0}>
        <span class='text-sm text-muted-foreground'>
          {t('events.neededFor', { count: mirrors() })}
        </span>
      </Show>
    </div>
  );

  return (
    <Show when={children().length > 0} fallback={card()}>
      <Collapsible
        class='gap-2 flex flex-col'
        open={isOpen()}
        onOpenChange={setIsOpen}
      >
        <div class='gap-2 flex items-start'>
          <CollapsibleTrigger class='mt-0.5 shrink-0'>
            <IconChevronDown
              class={cn('transition-transform', {
                '-rotate-90': !isOpen(),
              })}
            />
          </CollapsibleTrigger>
          <div class='min-w-0 flex-1'>{card()}</div>
        </div>
        <Show when={!isOpen() && failedMessage()}>
          <span class='text-sm text-destructive pl-6 truncate'>
            {failedMessage()}
          </span>
        </Show>
        <CollapsibleContent>
          <div class='gap-3 pl-6 flex flex-col'>
            <For each={children()}>
              {(child) => (
                <ProgressNode payload={child} payloads={props.payloads} />
              )}
            </For>
          </div>
        </CollapsibleContent>
      </Collapsible>
    </Show>
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
