import { showWindow } from '@/platform/window';
import { logError } from '@/shared/log';

import { exposeWindowMethods } from './exposeWindowMethods';

export const setupApp = async () => {
  try {
    exposeWindowMethods();

    await showWindow().catch(logError);
  } catch (e) {
    logError(e);
    await showWindow().catch(logError);
    throw e;
  }
};
