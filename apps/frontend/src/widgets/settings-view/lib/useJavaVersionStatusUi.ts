import { createEffect, createSignal, onCleanup, type Accessor } from 'solid-js';

import type { JavaTestStatus } from '../model';

interface JavaVersionStatusUiProps {
  testingStatus: Accessor<JavaTestStatus>;
  onTest: () => Promise<void>;
}

export const LOADER_DELAY = 250;
export const SPIN_DURATION = 500;
export const COOL_DOWN_DURATION = 3000;

export const useJavaVersionStatusUi = (props: JavaVersionStatusUiProps) => {
  const [uiStatus, setUiStatus] = createSignal<JavaTestStatus>('idle');
  const [isCoolingDown, setIsCoolingDown] = createSignal(false);
  const [isSuccessSpinning, setIsSuccessSpinning] = createSignal(false);

  let loaderTimer: ReturnType<typeof setTimeout>;
  let spinTimer: ReturnType<typeof setTimeout>;
  let collDownTimer: ReturnType<typeof setTimeout>;

  createEffect(() => {
    const current = props.testingStatus();
    clearTimeout(loaderTimer);

    if (current === 'testing') {
      loaderTimer = setTimeout(() => setUiStatus('testing'), LOADER_DELAY);
    } else {
      setUiStatus(current || 'idle');
    }
  });

  const handleTriggerTest = async () => {
    const previousStatus = props.testingStatus();

    clearTimeout(spinTimer);
    clearTimeout(collDownTimer);
    setIsSuccessSpinning(false);

    await props.onTest();

    setIsCoolingDown(true);
    collDownTimer = setTimeout(
      () => setIsCoolingDown(false),
      COOL_DOWN_DURATION,
    );

    if (props.testingStatus() === previousStatus && previousStatus !== 'idle') {
      setIsSuccessSpinning(true);
      spinTimer = setTimeout(() => setIsSuccessSpinning(false), SPIN_DURATION);
    }
  };

  onCleanup(() => {
    clearTimeout(loaderTimer);
    clearTimeout(spinTimer);
    clearTimeout(collDownTimer);
  });

  return {
    uiStatus,
    isCoolingDown,
    isSuccessSpinning,
    handleTriggerTest,
  };
};
