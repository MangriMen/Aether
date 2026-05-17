import IconMdiPlay from '~icons/mdi/play';
import {
  createMemo,
  Match,
  splitProps,
  Switch,
  type Component,
} from 'solid-js';

import type { ButtonProps } from '@/shared/ui';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button } from '@/shared/ui';

import { useJavaVersionTesting } from '../../lib';

export type JavaVersionStatusBadgeProps = ButtonProps & {
  majorVersion?: string;
  path?: string;
  disabled?: boolean;
};

export const JavaVersionStatusBadge: Component<JavaVersionStatusBadgeProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'majorVersion',
    'path',
    'disabled',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const { test, testingStatus } = useJavaVersionTesting();

  const isDataValid = createMemo(
    () => local.majorVersion !== undefined && local.path !== undefined,
  );

  const handleClick = () => {
    if (local.majorVersion === undefined || local.path === undefined) {
      return;
    }

    test(local.majorVersion, local.path);
  };

  const variant = createMemo<ButtonProps['variant']>(() => {
    switch (testingStatus()) {
      case undefined:
      case 'testing':
        return 'secondary';
      case 'error':
        return 'destructive';
      case 'valid':
        return 'success';
      case 'version-mismatch':
        return 'warning';
    }
  });

  return (
    <Button
      class={cn('w-full', local.class)}
      variant={variant()}
      onClick={handleClick}
      loading={testingStatus() === 'testing'}
      disabled={local.disabled || !isDataValid()}
      leadingIcon={testingStatus() === undefined ? IconMdiPlay : undefined}
      size='sm'
      {...others}
    >
      <Switch>
        <Match when={testingStatus() === undefined}>
          {t('settings.java.test')}
        </Match>

        <Match when={testingStatus() === 'testing'}>
          {t('javaVersion.testing')}
        </Match>

        <Match when={testingStatus() === 'valid'}>
          {t('javaVersion.valid')}
        </Match>

        <Match when={testingStatus() === 'error'}>
          {t('javaVersion.error')}
        </Match>

        <Match when={testingStatus() === 'version-mismatch'}>
          {t('javaVersion.versionMismatch')}
        </Match>
      </Switch>
    </Button>
  );
};
