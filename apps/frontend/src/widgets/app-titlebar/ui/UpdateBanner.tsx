import {
  createMemo,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { SettingsTab, useSettingsSearchParams } from '@/entities/settings';
import { useCheckUpdate } from '@/entities/updates';
import { checkIsUpdateAvailable } from '@/entities/updates';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button, CombinedTooltip } from '@/shared/ui';

export type UpdateBannerProps = ComponentProps<'button'>;

export const UpdateBanner: Component<UpdateBannerProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslation();

  const update = useCheckUpdate();

  const isUpdateAvailable = createMemo(() =>
    update.data ? checkIsUpdateAvailable(update.data) : false,
  );

  const { open } = useSettingsSearchParams();

  const handleGoToUpdate = () => {
    open(SettingsTab.Update);
  };

  return (
    <Show when={isUpdateAvailable()}>
      <CombinedTooltip
        label={t('settings.goToUpdatePage')}
        as={Button}
        class={cn('mt-auto bg-primary/40 border-primary border', local.class)}
        onClick={handleGoToUpdate}
        size='sm'
        {...others}
      >
        {t('settings.updateAvailable')}
      </CombinedTooltip>
    </Show>
  );
};
