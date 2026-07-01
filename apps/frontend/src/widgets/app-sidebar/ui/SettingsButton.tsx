import type { PolymorphicProps } from '@kobalte/core';
import type { Component, ValidComponent } from 'solid-js';

import IconMdiCog from '~icons/mdi/cog';
import { createMemo } from 'solid-js';

import type { IconButtonProps } from '@/shared/ui';

import { useSettingsSearchParams } from '@/entities/settings';
import { useCheckUpdate } from '@/entities/updates';
import { checkIsUpdateAvailable } from '@/entities/updates';
import { UpdateBadge } from '@/features/update-badge';
import { useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton } from '@/shared/ui';

export type SettingsButtonProps<T extends ValidComponent = 'button'> =
  PolymorphicProps<T, IconButtonProps<T>>;

const SettingsButton: Component<SettingsButtonProps> = (props) => {
  const [{ t }] = useTranslation();

  const update = useCheckUpdate();

  const isUpdateAvailable = createMemo(() =>
    update.data ? checkIsUpdateAvailable(update.data) : false,
  );

  const { open: openSettings } = useSettingsSearchParams();

  const handleClick = () => {
    openSettings();
  };

  return (
    <CombinedTooltip
      label={
        isUpdateAvailable()
          ? `${t('settings.title')} (${t('settings.updateAvailable')})`
          : `${t('settings.title')}`
      }
      placement='right'
      as={IconButton}
      class='relative'
      variant='ghost'
      size='lg'
      onClick={handleClick}
      {...props}
    >
      <UpdateBadge class='absolute top-1 right-2' />
      <IconMdiCog />
    </CombinedTooltip>
  );
};

export default SettingsButton;
