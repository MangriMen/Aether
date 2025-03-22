import {
  refetchInstanceContent,
  useInstanceContent,
  type Instance,
} from '@/entities/instances';

import { cn } from '@/shared/lib';
import { useTranslate } from '@/shared/model';
import {
  createSignal,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { ContentControls } from './ContentControls';
import { ContentTable } from './ContentTable';
import { useNavigate } from '@solidjs/router';

export type ContentTabProps = ComponentProps<'div'> & {
  instance: Instance;
};

export const ContentTab: Component<ContentTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const navigate = useNavigate();

  const [{ t }] = useTranslate();

  const instanceContent = useInstanceContent(() => local.instance.id);

  const [search, setSearch] = createSignal<string | undefined>();

  const handleInstallContent = () => {
    const searchParams = new URLSearchParams({
      instance: local.instance.id,
    });

    return navigate(`/content?${searchParams.toString()}`);
  };

  return (
    <div class={cn('flex flex-col gap-4 p-1', local.class)} {...others}>
      <Show
        when={instanceContent.array !== undefined && instanceContent.array}
        fallback={
          <span class='mx-auto mt-20 text-lg text-muted-foreground'>
            {t('instance.noContent')}
          </span>
        }
      >
        {(items) => (
          <>
            <ContentControls
              contentsCount={items().length ?? 0}
              onSearch={setSearch}
              onInstallContent={handleInstallContent}
            />
            <ContentTable
              data={items()}
              refetch={() => refetchInstanceContent(local.instance.id)}
              searchQuery={search()}
              isLoading={instanceContent.loading}
              instanceId={local.instance.id}
              instancePath={local.instance.path}
            />
          </>
        )}
      </Show>
    </div>
  );
};
