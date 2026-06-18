import type { DialogRootProps } from '@kobalte/core/dialog';

import { makePersisted } from '@solid-primitives/storage';
import IconMdiFullscreen from '~icons/mdi/fullscreen';
import IconMdiFullscreenExit from '~icons/mdi/fullscreen-exit';
// import IconMdiMenu from '~icons/mdi/menu';
import { createSignal, type Component } from 'solid-js';

import type { SettingsTab } from '@/entities/settings';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  IconButton,
} from '@/shared/ui';

import { useSettingsSearchParams } from '../../../entities/settings/lib/useSettingsSearchParams';
import { IS_SETTINGS_MODAL_FULLSCREEN_KEY } from '../model';
import { SettingsView } from './SettingsView';

export type SettingsDialogProps = DialogRootProps;

export const SettingsDialog: Component<SettingsDialogProps> = (props) => {
  const { params, setParams } = useSettingsSearchParams();

  const [{ t }] = useTranslation();

  const handleChangeTab = (tab: SettingsTab) => {
    setParams({ tab });
  };

  const [isFullScreen, setIsFullScreen] = makePersisted(
    // eslint-disable-next-line solid/reactivity
    createSignal(false),
    {
      name: IS_SETTINGS_MODAL_FULLSCREEN_KEY,
    },
  );

  const toggleFullscreen = () => {
    setIsFullScreen((prev) => !prev);
  };

  return (
    <Dialog {...props}>
      <DialogContent
        class={cn('@container/tabs-root flex flex-col p-0 pt-6', {
          'modal-adaptive-bounds': !isFullScreen(),
          'size-full top-[calc(50%+20px)] max-w-[calc(100%-16px)] max-h-[calc(100%-60px)]':
            isFullScreen(),
        })}
        onEscapeKeyDown={(e) => e.preventDefault()}
        actions={
          <IconButton
            size='sm'
            variant='ghost'
            class='opacity-70 transition-opacity hover:opacity-100'
            icon={isFullScreen() ? IconMdiFullscreenExit : IconMdiFullscreen}
            onClick={toggleFullscreen}
          />
        }
      >
        <DialogHeader class='z-50 w-max flex-row gap-2 pl-6 pr-20'>
          <DialogTitle class='!mt-0 self-center'>
            {t('settings.title')}
          </DialogTitle>
        </DialogHeader>
        <SettingsView
          class='grow pl-6'
          tab={params().tab}
          onChangeTab={handleChangeTab}
        />
      </DialogContent>
    </Dialog>
  );
};
