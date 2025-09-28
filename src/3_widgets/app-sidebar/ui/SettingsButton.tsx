import type { PolymorphicProps } from '@kobalte/core';
import type { Component, ValidComponent } from 'solid-js';

import { Icon } from '@iconify-icon/solid';
import MdiCogIcon from '@iconify/icons-mdi/cog';
import { useNavigate } from '@solidjs/router';
import { createMemo, Show } from 'solid-js';

import type { IconButtonProps } from '@/shared/ui';

import { useCheckUpdate } from '@/entities/updates';
import { checkIsUpdateAvailable } from '@/entities/updates/model';
import { useTranslation } from '@/shared/model';
import { Badge, CombinedTooltip, IconButton } from '@/shared/ui';

export type SettingsButtonProps<T extends ValidComponent = 'button'> =
  PolymorphicProps<T, IconButtonProps<T>>;

const SettingsButton: Component<SettingsButtonProps> = (props) => {
  const navigate = useNavigate();
  const [{ t }] = useTranslation();

  const update = useCheckUpdate();

  const isUpdateAvailable = createMemo(() =>
    update.data ? checkIsUpdateAvailable(update.data) : false,
  );

  const handleClick = () => navigate('/settings');

  return (
    <CombinedTooltip
      label={
        isUpdateAvailable()
          ? `${t('settings.title')}${t('common.updateAvailable')}`
          : `${t('settings.title')}`
      }
      placement='right'
      as={IconButton}
      class='relative'
      variant='ghost'
      onClick={handleClick}
      {...props}
    >
      <Show when={isUpdateAvailable()}>
        <Badge class='absolute right-2 top-1 aspect-square size-2 p-0' />
      </Show>
      <Icon icon={MdiCogIcon} />
    </CombinedTooltip>
  );
};

export default SettingsButton;
