import type { Component, ComponentProps } from 'solid-js';

import { type RouteSectionProps } from '@solidjs/router';
import { createMemo, Match, splitProps, Switch } from 'solid-js';

import { useInstance, useInstanceDir } from '@/entities/instances';
import { useTranslation } from '@/shared/model';
import { DelayedShow, Separator, Skeleton } from '@/shared/ui';

import { Body } from './Body';
import { Header } from './Header';

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
    <div class='p-page flex size-full flex-col gap-2' {...others}>
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
