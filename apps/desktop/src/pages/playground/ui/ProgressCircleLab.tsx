import { ProgressCircle } from '../../../shared/ui';
import { ComponentShelf } from './ComponentShelf';

export const ProgressCircleLab = () => {
  return (
    <ComponentShelf title='Progress Circle'>
      <ProgressCircle value={75} />
    </ComponentShelf>
  );
};
