import { createSignal, onCleanup } from 'solid-js';

import { useInstallJava, useTestJava } from '@/entities/java';
import { logError } from '@/shared/lib';

export const useJavaVersionActions = () => {
  const installJava = useInstallJava();

  const convertJavaVersionToNum = (version: string): number | undefined => {
    try {
      const versionNum = parseInt(version);

      if (Number.isNaN(versionNum)) {
        logError(`Java version is not number ${versionNum}`);
        return;
      }

      return versionNum;
    } catch {
      return;
    }
  };

  const installRecommended = async (version: string) => {
    const versionNum = convertJavaVersionToNum(version);

    if (!versionNum) {
      return;
    }

    await installJava.mutateAsync(versionNum);
  };

  const [isTestingFailed, setIsTestingFailed] = createSignal<
    boolean | undefined
  >(undefined);
  let timerId = 0;

  const updateIsTestingFailed = (value: boolean) => {
    setIsTestingFailed(value);

    timerId = setTimeout(() => {
      setIsTestingFailed(undefined);
    }, 2000);
  };

  onCleanup(() => {
    clearTimeout(timerId);
  });

  const testJava = useTestJava();

  const test = async (version: string, path: string) => {
    const versionNum = convertJavaVersionToNum(version);

    if (!versionNum) {
      return;
    }

    try {
      const result = await testJava.mutateAsync({
        major_version: versionNum,
        path,
      });
      updateIsTestingFailed(!result);
    } catch {
      updateIsTestingFailed(true);
    }
  };

  return {
    installRecommended,
    test,
    isInstalling: () => installJava.isPending,
    isTesting: () => testJava.isPending,
    isTestingFailed: () => isTestingFailed(),
  };
};
