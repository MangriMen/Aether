import { useInstallJava } from '@/entities/java';

import { parseJavaVersion } from './parseJavaVersion';

export const useJavaVersionActions = () => {
  const installJava = useInstallJava();

  const installRecommended = async (version: string) => {
    const versionNum = parseJavaVersion(version);

    if (!versionNum) {
      return;
    }

    await installJava.mutateAsync(versionNum);
  };

  return {
    installRecommended,
    isInstalling: () => installJava.isPending,
  };
};
