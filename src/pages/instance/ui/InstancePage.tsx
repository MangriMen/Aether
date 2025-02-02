import { type RouteSectionProps } from '@solidjs/router';
import type { Component, ComponentProps } from 'solid-js';
import { createMemo, Show, splitProps } from 'solid-js';

import { Separator } from '@/shared/ui';

import { useInstance } from '@/entities/instances';

import { Header } from './Header';
import { Body } from './Body';

export type InstancePageProps = ComponentProps<'div'> & RouteSectionProps;

export const InstancePage: Component<InstancePageProps> = (props) => {
  const [local, others] = splitProps(props, [
    'params',
    'location',
    'data',
    'children',
  ]);

  const id = createMemo(() => decodeURIComponent(props.params.id));

  // eslint-disable-next-line solid/reactivity
  const instance = useInstance(id);

  return (
    <div class='flex size-full flex-col gap-2 p-4' {...others}>
      <Show when={instance()} fallback={<span>Instance not found</span>}>
        {(instance) => (
          <>
            <Header instance={instance()} />
            <Separator />
            <Body instance={instance()} />
          </>
        )}
      </Show>
      {local.children}
    </div>
  );
};
