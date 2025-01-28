import MdiCogIcon from '@iconify/icons-mdi/cog';
import { Icon } from '@iconify-icon/solid';
import type { PolymorphicProps } from '@kobalte/core';
import { useNavigate } from '@solidjs/router';
import { Show } from 'solid-js';
import type { Component, ValidComponent } from 'solid-js';

import type { IconButtonProps } from '@/shared/ui';
import { Badge, CombinedTooltip, IconButton } from '@/shared/ui';

import { updateResource } from '@/entities/update';


import { useTranslate } from '@/app/model';

export type SettingsButtonProps<T extends ValidComponent = 'button'> =
  PolymorphicProps<T, IconButtonProps<T>>;

const SettingsButton: Component<SettingsButtonProps> = (props) => {
  const navigate = useNavigate();
  const [{ t }] = useTranslate();

  const [update] = updateResource;

  const handleClick = () => navigate('/settings');

  return (
    <CombinedTooltip
      label={
        update()?.available
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
      <Show when={update()?.available}>
        <Badge class='absolute right-2 top-1 aspect-square size-2 p-0' />
      </Show>
      <Icon icon={MdiCogIcon} />
    </CombinedTooltip>
  );
};

export default SettingsButton;
