import { useNavigate } from '@solidjs/router';
import {
  type Component,
  type ComponentProps,
  createMemo,
  createSignal,
  Show,
  splitProps,
} from 'solid-js';

import {
  ContentType,
  type Instance,
  InstanceInstallStage,
  useInstanceContents,
} from '@/entities/instances';
import { ModLoader } from '@/entities/minecraft';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import { ContentControls } from './ContentControls';
import { ContentTable } from './ContentTable';
import { InstallContentButton } from './InstallContentButton';

export type ContentTabProps = {
  instance: Instance;
  instancePath?: string;
} & ComponentProps<'div'>;

export const ContentTab: Component<ContentTabProps> = (props) => {
  const [local, others] = splitProps(props, [
    'instance',
    'instancePath',
    'class',
  ]);

  const navigate = useNavigate();

  const [{ t }] = useTranslation();

  const isInstalling = createMemo(
    () => local.instance.installStage !== InstanceInstallStage.Installed,
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

    return navigate(`/content?${searchParams.toString()}`);
  };

  const availableContent = createMemo(() => {
    if (!local.instance) {
      return [];
    } else if (local.instance.loader == ModLoader.Vanilla) {
      return [ContentType.ResourcePack, ContentType.DataPack];
    } else {
      return undefined;
    }
  });

  return (
    <div class={cn('flex flex-col gap-4 p-1', local.class)} {...others}>
      <Show
        fallback={
          <div class='mx-auto mt-20 flex flex-col items-center gap-4'>
            <span class='text-lg text-muted-foreground'>
              {t('instance.noContent')}
            </span>
            <InstallContentButton
              contentTypes={availableContent()}
              disabled={isInstalling()}
              instanceId={local.instance.id}
              onInstallContentClick={handleInstallContent}
            />
          </div>
        }
        when={
          instanceContentArray() !== undefined &&
          !!instanceContentArray()?.length &&
          instanceContentArray()
        }
      >
        {(items) => (
          <>
            <ContentControls
              contentsCount={items().length ?? 0}
              contentTypes={availableContent()}
              instanceId={local.instance.id}
              isInstalling={isInstalling()}
              onInstallContentClick={handleInstallContent}
              onSearch={setSearch}
            />
            <ContentTable
              data={items()}
              instanceId={local.instance.id}
              instancePath={local.instancePath}
              isLoading={instanceContent.isLoading}
              refetch={() => instanceContent.refetch()}
              searchQuery={search()}
            />
          </>
        )}
      </Show>
    </div>
  );
};
