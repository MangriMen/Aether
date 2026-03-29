import { type RouteSectionProps } from '@solidjs/router';
import {
  createMemo,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import type { ContentFilters } from '@/entities/instances';

import { useContentProviders, useInstance } from '@/entities/instances';
import { Separator } from '@/shared/ui';

import { useContentPageSearchParams } from '../lib/useContentFilters';
import { useContentPageState } from '../lib/useContentPageState';
import { contentProvidersToOptions } from '../model';
import { ContentBrowser } from './ContentBrowser';
import { ContentContextProvider } from './ContentContextProvider';
import { ContentPageInstanceInfo } from './ContentPageInstanceInfo';

export type ContentPageProps = ComponentProps<'div'> & RouteSectionProps;

export const ContentPage: Component<ContentPageProps> = (props) => {
  const [_, others] = splitProps(props, [
    'params',
    'location',
    'data',
    'children',
  ]);

  const [params, setParams] = useContentPageSearchParams();

  const contentProviders = useContentProviders();
  const contentProvidersOptions = createMemo(
    () => contentProvidersToOptions(contentProviders.data) ?? [],
  );

  const instance = useInstance(() => params().instanceId);

  const contentPageState = useContentPageState(
    () => instance.data,
    () => params(),
  );

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
    >
      <div class='flex size-full flex-col gap-2 p-4' {...others}>
        <Show when={contentPageState().isInstanceContentPage}>
          <>
            <ContentPageInstanceInfo instance={instance.data} />
            <Separator />
          </>
        </Show>
        <ContentBrowser
          providers={contentProvidersOptions()}
          isProvidersLoading={contentProviders.isLoading}
          types={contentPageState().availableContentTypes}
          onFiltersChange={handleFiltersChange}
        />
      </div>
    </ContentContextProvider>
  );
};
