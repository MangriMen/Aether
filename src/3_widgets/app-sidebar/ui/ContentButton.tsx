import type { Component, ComponentProps } from 'solid-js';

import MdiPackage from '@iconify/icons-mdi/package';
import { useNavigate } from '@solidjs/router';

import { useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton } from '@/shared/ui';

export type ContentButtonProps = ComponentProps<'div'>;

export const ContentButton: Component<ContentButtonProps> = (props) => {
  const navigate = useNavigate();
  const [{ t }] = useTranslation();

  const handleClick = () => navigate('/content');

  return (
    <CombinedTooltip
      as={IconButton}
      icon={MdiPackage}
      label={t('content.title')}
      onClick={handleClick}
      placement='right'
      variant='ghost'
      {...props}
    />
  );
};
