import {
  useInstallContent,
  type ContentItemExtended,
  type InstallContentPayload,
} from '@/entities/instances';
import { cn } from '@/shared/lib';
import { Button, Image } from '@/shared/ui';
import {
  createMemo,
  createSignal,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';
import MdiDownload from '@iconify/icons-mdi/download';
import MdiCheck from '@iconify/icons-mdi/check';
import { useTranslation } from '@/shared/model';

export type ContentListItemProps = ComponentProps<'div'> & {
  item: ContentItemExtended;
  instanceId: string;
  gameVersion: string;
  loader?: string;
  provider?: string;
  onInstalled?: (providerData: ContentItemExtended['providerData']) => void;
};

export const ContentListItem: Component<ContentListItemProps> = (props) => {
  const [local, others] = splitProps(props, [
    'item',
    'instanceId',
    'gameVersion',
    'loader',
    'provider',
    'onInstalled',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const [isInstalling, setIsInstalling] = createSignal(false);

  const { mutateAsync: installContent } = useInstallContent();

  const handleInstallContent = async () => {
    if (!local.provider) {
      return;
    }

    const payload: InstallContentPayload = {
      gameVersion: local.gameVersion,
      loader: local.loader,
      contentType: local.item.contentType,
      contentVersion: undefined,
      provider: local.provider,
      providerData: local.item.providerData,
    };

    setIsInstalling(true);
    await installContent({ id: local.instanceId, payload });
    setIsInstalling(false);
    local.onInstalled?.(local.item.providerData);
  };

  const installButtonText = createMemo(() => {
    if (isInstalling()) {
      return t('common.installing');
    } else if (local.item.installed) {
      return t('common.installed');
    } else {
      return t('common.install');
    }
  });

  return (
    <div
      class={cn(
        'flex gap-2 border border-secondary rounded-lg p-3',
        local.class,
      )}
      {...others}
    >
      <Image
        class='aspect-square size-24'
        src={local.item.iconUrl || undefined}
      />
      <div class='flex flex-col text-muted-foreground'>
        <span class='text-lg font-bold text-foreground'>
          <a href={local.item.url} target='_blank'>
            {local.item.name}
          </a>{' '}
          <span class='text-base font-semibold text-muted-foreground'>
            by {local.item.author}
          </span>
        </span>
        <span>{local.item.description}</span>
      </div>
      <div class='ml-auto flex flex-col justify-end'>
        <Button
          class='px-3'
          leadingIcon={local.item.installed ? MdiCheck : MdiDownload}
          onClick={handleInstallContent}
          disabled={local.item.installed}
        >
          {installButtonText()}
        </Button>
      </div>
    </div>
  );
};
