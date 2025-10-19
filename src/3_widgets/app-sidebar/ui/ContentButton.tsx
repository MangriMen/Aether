import type { Component, ComponentProps } from 'solid-js';

import { useNavigate } from '@solidjs/router';
import IconMdiPackage from '~icons/mdi/package';

import { useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton } from '@/shared/ui';

export type ContentButtonProps = ComponentProps<'div'>;

export const ContentButton: Component<ContentButtonProps> = (props) => {
  const navigate = useNavigate();
  const [{ t }] = useTranslation();

  const handleClick = () => navigate('/content');

  return (
    <CombinedTooltip
      label={t('content.title')}
      placement='right'
      as={IconButton}
      variant='ghost'
      icon={IconMdiPackage}
      onClick={handleClick}
      {...props}
    />
  );
};
