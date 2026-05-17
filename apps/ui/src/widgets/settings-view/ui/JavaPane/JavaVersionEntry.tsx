import IconMdiClose from '~icons/mdi/close';
import IconMdiDownload from '~icons/mdi/download';
import IconMdiFileFindOutline from '~icons/mdi/file-find-outline';
import IconMdiMagnify from '~icons/mdi/magnify';
import IconMdiPlay from '~icons/mdi/play';
import {
  Match,
  Show,
  splitProps,
  Switch,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { useTranslation } from '@/shared/model';
import { Button, CombinedTextField, LabeledField } from '@/shared/ui';

export type JavaVersionEntryProps = ComponentProps<'div'> & {
  version: string;
  path: string;
  isInstalling?: boolean;
  isTesting?: boolean;
  isTestingFailed?: boolean;
  onPathChange?: (path: string) => void;
  onInstallRecommended?: () => void;
  onDetect?: () => void;
  onBrowse?: () => void;
  onTest?: () => void;
};

export const JavaVersionEntry: Component<JavaVersionEntryProps> = (props) => {
  const [local, others] = splitProps(props, [
    'path',
    'version',
    'isInstalling',
    'isTesting',
    'isTestingFailed',
    'onPathChange',
    'onInstallRecommended',
    'onDetect',
    'onBrowse',
    'onTest',
  ]);

  const [{ t }] = useTranslation();

  const handlePathChange = (path: string) => {
    local.onPathChange?.(path);
  };

  return (
    <LabeledField
      class='overflow-hidden rounded-md border bg-card/card p-4 text-lg'
      label={t('settings.java.location', { version: local.version })}
      {...others}
    >
      <div class='flex flex-col gap-2'>
        <CombinedTextField
          value={local.path}
          onChange={handlePathChange}
          inputProps={{ type: 'text', placeholder: '/path/to/java' }}
          readOnly={
            local.onDetect === undefined || local.onBrowse === undefined
          }
        />
        <div class='flex flex-wrap gap-2'>
          <Button
            class='overflow-ellipsis whitespace-nowrap'
            variant='secondary'
            leadingIcon={IconMdiDownload}
            onClick={local.onInstallRecommended}
            loading={local.isInstalling}
          >
            {t('settings.java.installRecommended')}
          </Button>
          <Show when={local.onDetect !== undefined}>
            <Button
              variant='secondary'
              leadingIcon={IconMdiMagnify}
              onClick={local.onDetect}
            >
              {t('settings.java.detect')}
            </Button>
          </Show>
          <Show when={local.onBrowse !== undefined}>
            <Button
              variant='secondary'
              leadingIcon={IconMdiFileFindOutline}
              onClick={local.onBrowse}
            >
              {t('common.browse')}
            </Button>
          </Show>
          <Button
            variant='secondary'
            leadingIcon={local.isTestingFailed ? IconMdiClose : IconMdiPlay}
            onClick={local.onTest}
            loading={local.isInstalling || local.isTesting}
            disabled={local.isTestingFailed !== undefined}
          >
            <Switch>
              <Match when={local.isTestingFailed === undefined}>
                {t('settings.java.test')}
              </Match>
              <Match when={local.isTestingFailed === false}>
                {t('settings.java.testSuccess')}
              </Match>
              <Match when={local.isTestingFailed === true}>
                {t('settings.java.testFailed')}
              </Match>
            </Switch>
          </Button>
        </div>
      </div>
      {/* <div class='text-base'>Java Version</div> */}
    </LabeledField>
  );
};
