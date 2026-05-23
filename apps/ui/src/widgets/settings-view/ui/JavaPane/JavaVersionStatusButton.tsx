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
import { Button, CombinedTooltip } from '@/shared/ui';

import type { JavaTestStatus } from '../../model';

import { useJavaVersionTesting } from '../../lib';

export type JavaVersionStatusButtonProps = ButtonProps & {
  majorVersion: number;
  path?: string;
  isInstalling?: boolean;
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
const SPIN_DURATION = 500;
const COOLING_OFF_DURATION = 3000;

export const JavaVersionStatusButton: Component<
  JavaVersionStatusButtonProps
> = (props) => {
  const [local, others] = splitProps(props, [
    'majorVersion',
    'path',
    'isInstalling',
    'disabled',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const { test, testingStatus } = useJavaVersionTesting();

  const [uiStatus, setUiStatus] = createSignal(testingStatus());

  const [isCoolingOff, setIsCoolingOff] = createSignal(false);
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

  const isDataValid = createMemo(() => Boolean(local.path));

  let sameStatusTimer: number = 0;
  let coolingOffPeriodTimer: number = 0;

  const handleClick = async () => {
    if (!local.majorVersion || !local.path) return;

    const previousStatus = testingStatus();

    await test(local.majorVersion, local.path);

    setIsCoolingOff(true);
    coolingOffPeriodTimer = setTimeout(() => {
      setIsCoolingOff(false);
    }, COOLING_OFF_DURATION);

    if (testingStatus() === previousStatus && previousStatus !== 'idle') {
      setIsSuccessSpinning(true);

      sameStatusTimer = setTimeout(() => {
        setIsSuccessSpinning(false);
      }, SPIN_DURATION);
    }
  };

  onCleanup(() => {
    clearTimeout(coolingOffPeriodTimer);
    clearTimeout(sameStatusTimer);
  });

  const variant = createMemo(() => VARIANT_MAP[displayStatus()] || 'secondary');

  const leadingIcon = createMemo(() => {
    if (displayStatus() === 'idle') {
      return IconMdiPlay;
    }

    return () => (
      <IconMdiReload
        class={cn({
          [`animate-spin`]: isSuccessSpinning(),
        })}
        style={{ 'animation-duration': `${SPIN_DURATION}ms` }}
      />
    );
  });

  const isDisabled = createMemo(
    () =>
      local.disabled ||
      local.isInstalling ||
      testingStatus() === 'testing' ||
      isCoolingOff() ||
      !isDataValid(),
  );

  const tooltipLabel = createMemo(() => {
    if (local.isInstalling) {
      return t('javaVersion.installing');
    } else if (!isDataValid()) {
      return t('javaVersion.emptyPath');
    } else {
      return undefined;
    }
  });

  return (
    <CombinedTooltip
      label={tooltipLabel()}
      disableTooltip={!tooltipLabel()}
      as={Button}
      class={cn('w-full', local.class)}
      variant={variant()}
      onClick={handleClick}
      loading={uiStatus() === 'testing'}
      disabled={isDisabled()}
      leadingIcon={leadingIcon()}
      {...others}
    >
      {t(TEXT_MAP[displayStatus()])}
    </CombinedTooltip>
  );
};
