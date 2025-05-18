import { type RouteSectionProps } from '@solidjs/router';
import type { Component, ComponentProps } from 'solid-js';
import { createMemo, Show, splitProps } from 'solid-js';

import { Separator } from '@/shared/ui';

import { useInstance, useInstanceDir } from '@/entities/instances';

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

  const instance = useInstance(() => id());
  const instancePath = useInstanceDir(() => id());

  return (
    <div class='flex size-full flex-col gap-2 p-4' {...others}>
      <Show when={instance.data} fallback={<span>Instance not found</span>}>
        {(instance) => (
          <>
            <Header instance={instance()} instancePath={instancePath.data} />
            <Separator />
            <Body instance={instance()} instancePath={instancePath.data} />
          </>
        )}
      </Show>
      {local.children}
    </div>
  );
};
