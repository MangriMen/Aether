import { DiskUsage } from '@/app/shell';

import { ComponentShelf } from './ComponentShelf';

const MB = 1024 * 1024;

export const DiskUsageLab = () => (
  <>
    <ComponentShelf title='Disk usage: typical'>
      <div class='w-full'>
        <DiskUsage
          usage={{
            entries: [
              { kind: 'minecraft', bytes: 1369 * MB },
              { kind: 'java', bytes: 550 * MB },
              { kind: 'instances', bytes: 3400 * MB },
            ],
          }}
        />
      </div>
    </ComponentShelf>
    <ComponentShelf title='Disk usage: no instances yet'>
      <div class='w-full'>
        <DiskUsage
          usage={{
            entries: [
              { kind: 'minecraft', bytes: 1369 * MB },
              { kind: 'java', bytes: 550 * MB },
              { kind: 'instances', bytes: 0 },
            ],
          }}
        />
      </div>
    </ComponentShelf>
  </>
);
