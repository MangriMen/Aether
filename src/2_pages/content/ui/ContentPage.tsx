import {
  type RouteSectionProps,
  useNavigate,
  useSearchParams,
} from '@solidjs/router';
import {
  type Component,
  type ComponentProps,
  createMemo,
  Show,
  splitProps,
} from 'solid-js';

import {
  ContentType,
  useContentProviders,
  useInstance,
} from '@/entities/instances';
import { ModLoader } from '@/entities/minecraft';
import { Separator } from '@/shared/ui';

import { ContentBrowser } from './ContentBrowser';
import { InstanceInfo } from './InstanceInfo';

export type ContentPageProps = ComponentProps<'div'> & RouteSectionProps;

export const ContentPage: Component<ContentPageProps> = (props) => {
  const [_, others] = splitProps(props, [
    'params',
    'location',
    'data',
    'children',
  ]);

  const navigate = useNavigate();

  const [searchParams] = useSearchParams();

  const id = createMemo(() => {
    const instance = searchParams['instance'];

    if (instance === undefined || typeof instance !== 'string') {
      navigate('/');
      return '';
    }

    return decodeURIComponent(instance);
  });

  const instance = useInstance(() => id());

  const contentProviders = useContentProviders();

  const transformedContentProviders = createMemo(() =>
    contentProviders.data
      ? Object.entries(contentProviders.data).map(([key, value]) => ({
          name: key,
          value,
        }))
      : [],
  );

  const availableContent = createMemo(() => {
    if (!instance.data) {
      return [];
    } else if (instance.data.loader == ModLoader.Vanilla) {
      return [ContentType.ResourcePack, ContentType.DataPack];
    } else {
      return undefined;
    }
  });

  return (
    <div class='flex size-full flex-col gap-2 p-4' {...others}>
      <Show when={instance.data}>
        {(instance) => (
          <>
            <InstanceInfo instance={instance()} />
            <Separator />
            <ContentBrowser
              contentTypes={availableContent()}
              instance={instance()}
              providers={transformedContentProviders() ?? []}
            />
          </>
        )}
      </Show>
    </div>
  );
};
