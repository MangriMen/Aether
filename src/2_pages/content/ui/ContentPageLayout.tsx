import { type RouteSectionProps } from '@solidjs/router';
import {
  createMemo,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import type { ContentFilters } from '@/entities/instances';

import { useInstance } from '@/entities/instances';
import { Separator } from '@/shared/ui';

import {
  getFiltersFromInstance,
  getFiltersFromSearchParams,
  mergeContentFilters,
} from '../lib';
import { useContentPageSearchParams } from '../lib/useContentFilters';
import { ContentContextProvider } from './ContentContextProvider';
import { ContentPageInstanceInfo } from './ContentPageInstanceInfo';

export type ContentPageProps = ComponentProps<'div'> & RouteSectionProps;

export const ContentPageLayout: Component<ContentPageProps> = (props) => {
  const [local, others] = splitProps(props, [
    'params',
    'location',
    'data',
    'children',
  ]);

  const [params, setParams] = useContentPageSearchParams();

  const instance = useInstance(() => params().instanceId);

  const isInstancePage = createMemo(() => Boolean(params().instanceId));

  const contentPageState = createMemo(() => {
    const instanceData = instance.data;
    const paramsData = params();

    const { filters, filtersLock } = mergeContentFilters(
      getFiltersFromInstance(instanceData),
      getFiltersFromSearchParams(paramsData),
    );

    return {
      filters,
      filtersLock,
    };
  });

  const handleFiltersChange = (filters: ContentFilters) => {
    setParams({
      ...filters,
      instanceId: params().instanceId,
    });
  };

  return (
    <ContentContextProvider
      instanceId={params().instanceId}
      providerId={params().providerId}
      filters={contentPageState().filters}
      filtersLock={contentPageState().filtersLock}
      onFiltersChange={handleFiltersChange}
    >
      <div
        class='p-page flex size-full flex-col gap-2 overflow-hidden'
        {...others}
      >
        <Show when={isInstancePage()}>
          <>
            <ContentPageInstanceInfo instance={instance.data} />
            <Separator />
          </>
        </Show>
        {local.children}
      </div>
    </ContentContextProvider>
  );
};
