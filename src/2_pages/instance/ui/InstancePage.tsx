import { type RouteSectionProps } from '@solidjs/router';
import type { Component, ComponentProps } from 'solid-js';
import { createMemo, Match, splitProps, Switch } from 'solid-js';

import { DelayedShow, Separator, Skeleton } from '@/shared/ui';

import { useInstance, useInstanceDir } from '@/entities/instances';

import { Header } from './Header';
import { Body } from './Body';
import { useTranslation } from '@/shared/model';

export type InstancePageProps = ComponentProps<'div'> & RouteSectionProps;

export const InstancePage: Component<InstancePageProps> = (props) => {
  const [local, others] = splitProps(props, [
    'params',
    'location',
    'data',
    'children',
  ]);

  const id = createMemo(() => decodeURIComponent(props.params.id));

  const [{ t }] = useTranslation();

  const instance = useInstance(() => id());
  const instancePath = useInstanceDir(() => id());

  return (
    <div class='flex size-full flex-col gap-2 p-4' {...others}>
      <DelayedShow
        when={instance.data}
        fallback={
          <Switch>
            <Match when={instance.isLoading}>
              <Skeleton class='w-full' height={96} radius={8} />
              <Skeleton class='grow' radius={8} />
            </Match>
            <Match when={instance.isError}>
              <h2 class='flex size-full flex-col items-center justify-center text-xl font-semibold'>
                {t('instance.notFound', { id: id() })}
              </h2>
            </Match>
          </Switch>
        }
      >
        {(instance) => (
          <>
            <Header instance={instance()} instancePath={instancePath.data} />
            <Separator />
            <Body instance={instance()} instancePath={instancePath.data} />
          </>
        )}
      </DelayedShow>
      {local.children}
    </div>
  );
};
