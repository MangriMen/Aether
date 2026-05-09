import { Progress, ProgressLabel, ProgressValueLabel } from '@/shared/ui';

import { ComponentShelf } from './ComponentShelf';

export const ProgressLab = () => {
  return (
    <ComponentShelf title='Progress'>
      <Progress
        value={3}
        minValue={0}
        maxValue={10}
        getValueLabel={({ value, max }) => `${value} of ${max} tasks completed`}
        class='w-[300px] space-y-1'
      >
        <div class='flex justify-between'>
          <ProgressLabel>Processing...</ProgressLabel>
          <ProgressValueLabel />
        </div>
      </Progress>
    </ComponentShelf>
  );
};
