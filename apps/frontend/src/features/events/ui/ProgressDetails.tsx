import {
  For,
  splitProps,
  type Accessor,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { cn } from '@/shared/utils';

import { type ProgressEvent } from '../model/progressEvent';
import { ProgressCard } from './ProgressCard';

export type ProgressDetailsProps = ComponentProps<'div'> & {
  payloads: Accessor<ProgressEvent[]>;
};

export const ProgressDetails: Component<ProgressDetailsProps> = (props) => {
  const [local, others] = splitProps(props, ['payloads', 'class']);

  return (
    <div
      class={cn(
        'gap-3 flex flex-col overflow-x-hidden overflow-y-scroll',
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
