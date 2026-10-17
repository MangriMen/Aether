import { isDebug } from './debug';

export const logError = (...data: unknown[]) => {
  if (isDebug()) {
    // eslint-disable-next-line no-console
    console.error(...data);
  }
};

export const logDebug = (...data: unknown[]) => {
  if (isDebug()) {
    // eslint-disable-next-line no-console
    console.debug(...data);
  }
};
