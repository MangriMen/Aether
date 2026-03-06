import { useNavigate } from '@solidjs/router';
import IconMdiArrowLeft from '~icons/mdi/arrow-left';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import type { Instance } from '@/entities/instances';

import { InstanceGameVersion } from '@/entities/instances';
import { ROUTES } from '@/shared/config';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button, DelayedShow, Image, Skeleton } from '@/shared/ui';

export type InstanceInfoProps = ComponentProps<'div'> & {
  instance?: Instance;
};

export const InstanceInfo: Component<InstanceInfoProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const navigate = useNavigate();

  const [{ t }] = useTranslation();

  const handleBackToInstance = () => {
    if (!local.instance?.id) {
      return;
    }

    navigate(ROUTES.INSTANCE(local.instance.id));
  };

  return (
    <div class={cn('flex gap-2 items-center', local.class)} {...others}>
      <DelayedShow
        when={local.instance}
        fallback={<Skeleton class='p-1' width={48} height={48} radius={4} />}
      >
        <Image class='h-14 w-max' />
      </DelayedShow>
      <div class='flex flex-col justify-between text-muted-foreground'>
        <DelayedShow
          when={local.instance?.name}
          fallback={<Skeleton class='mb-1' width={96} height={20} radius={4} />}
        >
          {(name) => (
            <span class='line-clamp-2 text-lg font-bold text-foreground'>
              {name()}
            </span>
          )}
        </DelayedShow>
        <DelayedShow
          when={local.instance}
          fallback={<Skeleton width={128} height={16} radius={4} />}
        >
          {(instance) => (
            <InstanceGameVersion
              class='font-medium'
              loader={instance().loader}
              gameVersion={instance().gameVersion}
            />
          )}
        </DelayedShow>
      </div>
      <Button
        class='ml-auto min-w-max'
        leadingIcon={IconMdiArrowLeft}
        variant='secondary'
        onClick={handleBackToInstance}
        loading={!local.instance}
      >
        {t('content.backToInstance')}
      </Button>
    </div>
  );
};
