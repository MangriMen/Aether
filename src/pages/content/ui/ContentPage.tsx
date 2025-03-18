import {
  useNavigate,
  useSearchParams,
  type RouteSectionProps,
} from '@solidjs/router';
import {
  createMemo,
  createResource,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';
import { InstanceInfo } from './InstanceInfo';
import {
  ContentType,
  getContentProviders,
  useInstance,
} from '@/entities/instances';
import { Separator } from '@/shared/ui';
import { ContentBrowser } from './ContentBrowser';
import { ModLoader } from '@/entities/minecrafts';

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

  // eslint-disable-next-line solid/reactivity
  const instance = useInstance(id);

  const getTransformedContentProviders = async () => {
    const res = await getContentProviders();

    return Object.entries(res).map(([key, value]) => ({
      name: key,
      value,
    }));
  };

  const [contentProviders] = createResource(getTransformedContentProviders, {
    initialValue: [],
  });

  const availableContent = createMemo(() => {
    const inst = instance();

    if (!inst) {
      return [];
    } else if (inst.loader == ModLoader.Vanilla) {
      return [ContentType.ResourcePack, ContentType.DataPack];
    } else {
      return undefined;
    }
  });

  return (
    <div class='flex size-full flex-col gap-2 p-4' {...others}>
      <Show when={instance()}>
        {(instance) => (
          <>
            <InstanceInfo instance={instance()} />
            <Separator />
            <ContentBrowser
              instance={instance()}
              providers={contentProviders() ?? []}
              availableContent={availableContent()}
            />
          </>
        )}
      </Show>
    </div>
  );
};
