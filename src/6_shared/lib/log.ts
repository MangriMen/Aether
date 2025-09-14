import { isDebug } from '../model';

export const debugError = (...data: unknown[]) => {
  if (isDebug()) {
    // eslint-disable-next-line no-console
    console.error(...data);
  }
};

export const debugLog = (...data: unknown[]) => {
  if (isDebug()) {
    // eslint-disable-next-line no-console
    console.log(...data);
  }
};
