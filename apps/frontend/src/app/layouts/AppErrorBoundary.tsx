import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button } from '@/shared/ui';

export type AppErrorBoundaryProps = ComponentProps<'div'> & {
  reset: () => void;
};

export const AppErrorBoundary: Component<AppErrorBoundaryProps> = (props) => {
  const [local, others] = splitProps(props, ['reset', 'class']);

  const [{ t }] = useTranslation();

  return (
    <div
      class={cn(
        'gap-5 flex size-full flex-col items-center justify-center',
        local.class,
      )}
      {...others}
    >
      <h1 class='text-center whitespace-pre-line'>{t('common.appError')}</h1>
      <Button onClick={props.reset}>{t('common.reset')}</Button>
    </div>
  );
};
