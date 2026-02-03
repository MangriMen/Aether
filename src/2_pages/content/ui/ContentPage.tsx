import {
  useNavigate,
  useSearchParams,
  type RouteSectionProps,
} from '@solidjs/router';
import {
  createMemo,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import type { ContentProviderCapabilityMetadata } from '@/entities/instances';
import type { CapabilityEntry } from '@/shared/model';

import {
  ContentType,
  useContentProviders,
  useInstance,
} from '@/entities/instances';
import { ModLoader } from '@/entities/minecraft';
import { ROUTES } from '@/shared/config';
import { type Option } from '@/shared/model';
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
      navigate(ROUTES.HOME);
      return '';
    }

    return decodeURIComponent(instance);
  });

  const instance = useInstance(() => id());

  const contentProviders = useContentProviders();

  const transformedContentProviders = createMemo<
    Option<CapabilityEntry<ContentProviderCapabilityMetadata>>[]
  >(() =>
    contentProviders.data
      ? contentProviders.data.map((value) => ({
          name: value.capability.name,
          value: value,
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
              instance={instance()}
              providers={transformedContentProviders() ?? []}
              contentTypes={availableContent()}
            />
          </>
        )}
      </Show>
    </div>
  );
};
