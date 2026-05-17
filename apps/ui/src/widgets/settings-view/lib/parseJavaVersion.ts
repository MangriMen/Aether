import { logError } from '@/shared/lib';

export const parseJavaVersion = (version: string): number | undefined => {
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
