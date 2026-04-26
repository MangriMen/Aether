import { useNavigate } from '@solidjs/router';
import IconMdiChevronLeft from '~icons/mdi/chevron-left';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { ROUTES } from '@/shared/config';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button } from '@/shared/ui';

export type GoHomeFromPlaygroundButtonProps = ComponentProps<'div'>;

export const GoHomeFromPlaygroundButton: Component<
  GoHomeFromPlaygroundButtonProps
> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslation();

  const navigate = useNavigate();

  const handleClick = () => {
    navigate(ROUTES.HOME);
  };

  return (
    <div class={cn('px-1', local.class)} {...others}>
      <Button size='sm' variant='secondary' onClick={handleClick}>
        <IconMdiChevronLeft />
        {t('home.title')}
      </Button>
    </div>
  );
};
