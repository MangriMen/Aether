import IconMdiChevronRight from '~icons/mdi/chevron-right';
// import IconMdiMenu from '~icons/mdi/menu';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { type Instance } from '@/entities/instances';
import { useTranslation } from '@/shared/model';
import { DialogHeader, DialogTitle, Image } from '@/shared/ui';

export type InstanceSettingsDialogHeaderProps = ComponentProps<'div'> & {
  instance: Instance;
  isTabListOpen?: boolean;
  onTabListOpenChange?: (open: boolean) => void;
};

const InstanceSettingsDialogHeader: Component<
  InstanceSettingsDialogHeaderProps
> = (props) => {
  const [local, others] = splitProps(props, [
    'instance',
    'isTabListOpen',
    'onTabListOpenChange',
  ]);

  const [{ t }] = useTranslation();

  return (
    <DialogHeader {...others}>
      <DialogTitle class='flex items-center gap-2 pl-6 text-muted-foreground'>
        {/* <IconButton
          class='z-50'
          variant='ghost'
          icon={IconMdiMenu}
          onClick={toggleTabList}
        /> */}
        <div class='flex items-center gap-1'>
          <div class='flex items-center gap-2'>
            <Image
              class='size-8 min-w-max'
              src={local.instance.iconPath ?? undefined}
            />
            <span class='line-clamp-1 leading-8 [word-break:break-word]'>
              {local.instance.name}
            </span>
          </div>
          <IconMdiChevronRight />
          <span class='text-foreground'>{t('instance.settings')}</span>
        </div>
      </DialogTitle>
    </DialogHeader>
  );
};

export default InstanceSettingsDialogHeader;
