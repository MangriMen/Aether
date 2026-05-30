import { useNavigate } from '@solidjs/router';
import {
  createMemo,
  createSignal,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import {
  ContentType,
  useInstanceContents,
  type Instance,
} from '@/entities/instances';
import { ROUTES } from '@/shared/config';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { DelayedShow } from '@/shared/ui';

import { ContentControls } from './ContentControls';
import { ContentTable } from './ContentTable';
import { InstallContentButton } from './InstallContentButton';

export type ContentTabProps = ComponentProps<'div'> & {
  instance: Instance;
  instancePath?: string;
};

export const ContentTab: Component<ContentTabProps> = (props) => {
  const [local, others] = splitProps(props, [
    'instance',
    'instancePath',
    'class',
  ]);

  const navigate = useNavigate();

  const [{ t }] = useTranslation();

  const isInstalling = createMemo(
    () => local.instance.installStage !== 'installed',
  );

  const instanceContent = useInstanceContents(() => local.instance.id);
  const instanceContentArray = createMemo(() =>
    instanceContent.data ? Object.values(instanceContent.data) : undefined,
  );

  const [search, setSearch] = createSignal<string | undefined>();

  const handleInstallContent = () => {
    const searchParams = new URLSearchParams({
      instance: local.instance.id,
    });

    return navigate(`${ROUTES.CONTENT}?${searchParams.toString()}`);
  };

  const availableContent = createMemo(() => {
    if (!local.instance) {
      return [];
    } else if (local.instance.loader == 'vanilla') {
      return [ContentType.ResourcePack, ContentType.DataPack];
    } else {
      return undefined;
    }
  });

  return (
    <div class={cn('flex flex-col gap-4 p-1', local.class)} {...others}>
      <DelayedShow
        when={
          instanceContentArray() !== undefined &&
          !!instanceContentArray()?.length &&
          instanceContentArray()
        }
        delay={20}
        fallback={
          <div class='mx-auto mt-20 flex flex-col items-center gap-4'>
            <span class='text-lg text-muted-foreground'>
              {t('instance.noContent')}
            </span>
            <InstallContentButton
              instanceId={local.instance.id}
              onInstallContentClick={handleInstallContent}
              contentTypes={availableContent()}
              disabled={isInstalling()}
            />
          </div>
        }
      >
        {(items) => (
          <>
            <ContentControls
              instanceId={local.instance.id}
              contentsCount={items().length ?? 0}
              onSearch={setSearch}
              onInstallContentClick={handleInstallContent}
              contentTypes={availableContent()}
              isInstalling={isInstalling()}
            />
            <ContentTable
              data={items()}
              refetch={() => instanceContent.refetch()}
              searchQuery={search()}
              isLoading={instanceContent.isLoading}
              instanceId={local.instance.id}
              instancePath={local.instancePath}
            />
          </>
        )}
      </DelayedShow>
    </div>
  );
};
