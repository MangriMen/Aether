import { useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton } from '@/shared/ui';
import { useNavigate } from '@solidjs/router';
import type { Component, ComponentProps } from 'solid-js';
import MdiPackage from '@iconify/icons-mdi/package';

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
      icon={MdiPackage}
      onClick={handleClick}
      {...props}
    />
  );
};
