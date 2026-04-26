import { createMemo, type Component, type ComponentProps } from 'solid-js';

import { useContentProviders, useInstance } from '../../../entities/instances';
import { getAvailableContentTypes } from '../lib';
import { contentProvidersToOptions, useContentContext } from '../model';
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
