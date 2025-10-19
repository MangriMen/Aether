import IconMdiChevronRight from '~icons/mdi/chevron-right';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import type { Instance } from '@/entities/instances';

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

  return (
    <DialogHeader {...others}>
      <DialogTitle class='flex items-center gap-1 text-muted-foreground'>
        <div class='flex items-center gap-2'>
          <Image class='size-8' />
          <span>{local.instance.name}</span>
        </div>
        <IconMdiChevronRight class='text-xl' />
        <span class='text-foreground'>{t('instance.settings')}</span>
      </DialogTitle>
    </DialogHeader>
  );
};

export default InstanceSettingsDialogHeader;
