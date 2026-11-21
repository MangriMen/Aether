import type { ContentFileDto } from '@/platform/bindings/instance';

import { instanceCommands } from '@/platform/ipc';

type Contents = { [key in string]: ContentFileDto };
type Waiting = {
  resolve: (contents: Contents) => void;
  reject: (reason: unknown) => void;
};

let waiting = new Map<string, Waiting[]>();

const flush = async () => {
  const batch = waiting;
  waiting = new Map();

  try {
    const answer = await instanceCommands.listContents([...batch.keys()]);

    for (const [id, askers] of batch) {
      const contents = answer[id];

      for (const asker of askers) {
        if (contents) {
          asker.resolve(contents);
        } else {
          asker.reject(new Error(`Could not list the contents of ${id}`));
        }
      }
    }
  } catch (err) {
    for (const askers of batch.values()) {
      askers.forEach((asker) => asker.reject(err));
    }
  }
};

/**
 * The contents of an instance. The instances asked for in the same turn of the event loop are
 * listed by one call to the backend, each still answered on its own, so every instance keeps
 * its own query.
 */
export const listContentBatched = (id: string): Promise<Contents> =>
  new Promise((resolve, reject) => {
    if (waiting.size === 0) {
      setTimeout(() => void flush(), 0);
    }

    const asker = { resolve, reject };
    const askers = waiting.get(id);

    if (askers) {
      askers.push(asker);
    } else {
      waiting.set(id, [asker]);
    }
  });
