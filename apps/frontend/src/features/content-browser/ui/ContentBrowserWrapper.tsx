import { createMemo, type Component, type ComponentProps } from 'solid-js';

import { useContentContext, useContentProviders } from '@/features/content';
import { useInstance } from '@/features/instance';

import { contentProvidersToOptions } from '../model/contentProviderMapper';
import { getAvailableContentTypes } from '../model/getAvailableContentTypes';
import { ContentBrowser } from './ContentBrowser';

export const ContentBrowserWrapper: Component<ComponentProps<'div'>> = (
  props,
) => {
  const contentProviders = useContentProviders();
  const contentProvidersOptions = createMemo(
    () => contentProvidersToOptions(contentProviders.data) ?? [],
  );

  const [context] = useContentContext();

  const instance = useInstance(() => context.instanceId);

  const availableContentTypes = createMemo(() =>
    getAvailableContentTypes(
      instance.data?.loader,
      Boolean(context.instanceId),
    ),
  );

  return (
    <ContentBrowser
      providers={contentProvidersOptions()}
      isProvidersLoading={contentProviders.isLoading}
      types={availableContentTypes()}
      {...props}
    />
  );
};
