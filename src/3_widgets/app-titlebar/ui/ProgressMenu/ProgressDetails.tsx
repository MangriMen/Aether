import {
  type Accessor,
  type Component,
  type ComponentProps,
  For,
  splitProps,
} from 'solid-js';

import { type LoadingPayload, ProgressCard } from '@/entities/events';
import { cn } from '@/shared/lib';

export type ProgressDetailsProps = {
  payloads: Accessor<LoadingPayload[]>;
} & ComponentProps<'div'>;

export const ProgressDetails: Component<ProgressDetailsProps> = (props) => {
  const [local, others] = splitProps(props, ['payloads', 'class']);

  return (
    <div
      class={cn(
        'flex flex-col gap-3 overflow-x-hidden overflow-y-scroll',
        local.class,
      )}
      {...others}
    >
      <For each={local.payloads()}>
        {(payload) => <ProgressCard payload={payload} />}
      </For>
    </div>
  );
};
