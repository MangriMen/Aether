import {
  getInstanceContents,
  type Instance,
  type InstanceFile,
} from '@/entities/instances';

import { cn } from '@/shared/lib';
import { useTranslate } from '@/shared/model';
import {
  createResource,
  createSignal,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { ContentControls } from './ContentControls';
import { ContentTable } from './ContentTable';

export type ContentTabProps = ComponentProps<'div'> & {
  instance: Instance;
};

export const ContentTab: Component<ContentTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const [{ t }] = useTranslate();

  const [contents, { refetch }] = createResource(
    () => local.instance,
    (instance) =>
      getInstanceContents(instance.id)
        .then((contentsObj) => Object.values<InstanceFile>(contentsObj))
        .catch(() => []),
    { initialValue: [] },
  );

  const [search, setSearch] = createSignal<string | undefined>();

  return (
    <div class={cn('flex flex-col gap-4 p-1', local.class)} {...others}>
      <Show
        when={contents.loading || contents()}
        fallback={
          <span class='mx-auto mt-20 text-lg text-muted-foreground'>
            {t('instance.noContent')}
          </span>
        }
      >
        <ContentControls
          contentsCount={contents().length}
          onSearch={setSearch}
        />
        <ContentTable
          data={contents()}
          refetch={() => refetch()}
          searchQuery={search()}
          isLoading={contents.loading}
          instanceId={local.instance.id}
          instancePath={local.instance.path}
        />
      </Show>
    </div>
  );
};
