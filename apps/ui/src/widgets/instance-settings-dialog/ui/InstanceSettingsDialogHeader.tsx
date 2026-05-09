import IconMdiChevronRight from '~icons/mdi/chevron-right';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { useInstanceIconSrc, type Instance } from '@/entities/instances';
import { useTranslation } from '@/shared/model';
import { DialogHeader, DialogTitle, Image } from '@/shared/ui';

export type InstanceSettingsDialogHeaderProps = ComponentProps<'div'> & {
  instance: Instance;
};

const InstanceSettingsDialogHeader: Component<
  InstanceSettingsDialogHeaderProps
> = (props) => {
  const [local, others] = splitProps(props, ['instance']);

  const [{ t }] = useTranslation();

  const iconSrc = useInstanceIconSrc(() => local.instance);

  return (
    <DialogHeader {...others}>
      <DialogTitle class='flex items-center gap-1 text-muted-foreground'>
        <div class='flex items-center gap-2'>
          <Image class='size-8 min-w-max' src={iconSrc()} />
          <span class='line-clamp-1 leading-8 [word-break:break-word]'>
            {local.instance.name}
          </span>
        </div>
        <IconMdiChevronRight />
        <span class='text-foreground'>{t('instance.settings')}</span>
      </DialogTitle>
    </DialogHeader>
  );
};

export default InstanceSettingsDialogHeader;
