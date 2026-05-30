import type { Accessor } from 'solid-js';

import IconMdiPlay from '~icons/mdi/play';
import IconMdiReload from '~icons/mdi/reload';
import {
  createEffect,
  createMemo,
  Show,
  splitProps,
  type Component,
} from 'solid-js';

import type { ButtonProps } from '@/shared/ui';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button, CombinedTooltip } from '@/shared/ui';

import type { JavaTestStatus } from '../../model';

import { useJavaVersionTesting } from '../../lib';
import {
  SPIN_DURATION,
  useJavaVersionStatusUi,
} from '../../lib/useJavaVersionStatusUi';

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

  const { test, testingStatus, resetStatus } = useJavaVersionTesting();

  const ui = useJavaVersionStatusUi({
    testingStatus,
    onTest: () => {
      if (!local.majorVersion || !local.path) return Promise.resolve();
      return test(local.majorVersion, local.path);
    },
  });

  const isPathEmpty = createMemo(() => !local.path);

  createEffect(() => {
    if (isPathEmpty()) {
      resetStatus();
    }
  });

  const variant = createMemo(() => VARIANT_MAP[ui.uiStatus()] || 'secondary');

  const isDisabled = createMemo(
    () =>
      local.disabled ||
      local.isInstalling ||
      testingStatus() === 'testing' ||
      ui.isCoolingDown() ||
      isPathEmpty(),
  );

  const tooltipLabel = createMemo(() => {
    if (local.isInstalling) {
      return t('javaVersion.installing');
    } else if (isPathEmpty()) {
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
      onClick={ui.handleTriggerTest}
      loading={ui.uiStatus() === 'testing'}
      disabled={isDisabled()}
      leadingIcon={() => (
        <StatusIcon
          status={ui.uiStatus()}
          isSuccessSpinning={ui.isSuccessSpinning}
        />
      )}
      {...others}
    >
      {t(TEXT_MAP[ui.uiStatus()])}
    </CombinedTooltip>
  );
};

export type StatusIconProps = {
  status: JavaTestStatus;
  isSuccessSpinning: Accessor<boolean>;
};

export const StatusIcon: Component<StatusIconProps> = (props) => {
  return (
    <Show
      when={props.status === 'idle'}
      fallback={
        <IconMdiReload
          class={cn({
            [`animate-spin`]: props.isSuccessSpinning(),
          })}
          style={{ 'animation-duration': `${SPIN_DURATION}ms` }}
        />
      }
    >
      <IconMdiPlay />
    </Show>
  );
};
