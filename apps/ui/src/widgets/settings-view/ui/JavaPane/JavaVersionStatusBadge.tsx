import IconMdiPlay from '~icons/mdi/play';
import IconMdiReload from '~icons/mdi/reload';
import {
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  splitProps,
  type Component,
} from 'solid-js';

import type { ButtonProps } from '@/shared/ui';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button } from '@/shared/ui';

import type { JavaTestStatus } from '../../model';

import { useJavaVersionTesting } from '../../lib';

export type JavaVersionStatusBadgeProps = ButtonProps & {
  majorVersion?: string;
  path?: string;
  disabled?: boolean;
};

const VARIANT_MAP = {
  idle: undefined,
  testing: 'secondary',
  error: 'destructive',
  valid: 'success',
  'version-mismatch': 'warning',
} as const satisfies Record<JavaTestStatus, ButtonProps['variant']>;

const TEXT_MAP = {
  idle: 'javaVersion.test',
  testing: 'javaVersion.testing',
  valid: 'javaVersion.valid',
  error: 'javaVersion.error',
  'version-mismatch': 'javaVersion.versionMismatch',
} as const satisfies Record<JavaTestStatus, string>;

const LOADER_DELAY = 250;
const SPIN_DURATION = 300;

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

  const [uiStatus, setUiStatus] = createSignal(testingStatus());

  const [isSuccessSpinning, setIsSuccessSpinning] = createSignal(false);

  createEffect(() => {
    const currentStatus = testingStatus();

    if (currentStatus === 'testing') {
      const timer = setTimeout(() => {
        setUiStatus('testing');
      }, LOADER_DELAY);

      onCleanup(() => clearTimeout(timer));
    } else {
      setUiStatus(currentStatus);
    }
  });

  const displayStatus = createMemo(() => uiStatus() || 'idle');

  const isDataValid = createMemo(
    () => local.majorVersion !== undefined && local.path !== undefined,
  );

  const handleClick = async () => {
    if (!local.majorVersion || !local.path) return;

    const previousStatus = testingStatus();

    await test(local.majorVersion, local.path);

    if (testingStatus() === previousStatus && previousStatus !== 'idle') {
      setIsSuccessSpinning(true);

      const timer = setTimeout(() => {
        setIsSuccessSpinning(false);
      }, SPIN_DURATION);

      onCleanup(() => clearTimeout(timer));
    }
  };

  const variant = createMemo(() => VARIANT_MAP[displayStatus()] || 'secondary');

  const leadingIcon = createMemo(() => {
    if (displayStatus() === 'idle') {
      return IconMdiPlay;
    }

    return () => (
      <IconMdiReload
        class={cn({
          [`animate-spin duration-${SPIN_DURATION}`]: isSuccessSpinning(),
        })}
      />
    );
  });

  return (
    <Button
      class={cn('w-full', local.class)}
      variant={variant()}
      onClick={handleClick}
      loading={uiStatus() === 'testing'}
      disabled={
        local.disabled || !isDataValid() || testingStatus() === 'testing'
      }
      leadingIcon={leadingIcon()}
      size='sm'
      {...others}
    >
      {t(TEXT_MAP[displayStatus()])}
    </Button>
  );
};
