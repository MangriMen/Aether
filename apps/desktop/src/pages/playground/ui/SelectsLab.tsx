import { createSignal } from 'solid-js';

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../../../shared/ui';
import { ComponentShelf } from './ComponentShelf';

export const SelectLab = () => {
  const [value, setValue] = createSignal('');

  return (
    <ComponentShelf title='Select'>
      <Select
        value={value()}
        onChange={setValue}
        options={['Apple', 'Banana', 'Blueberry', 'Grapes', 'Pineapple']}
        placeholder='Select a fruit…'
        itemComponent={(props) => (
          <SelectItem item={props.item}>{props.item.rawValue}</SelectItem>
        )}
      >
        <SelectTrigger aria-label='Fruit' class='w-[180px]'>
          <SelectValue<string>>{(state) => state.selectedOption()}</SelectValue>
        </SelectTrigger>
        <SelectContent />
      </Select>
    </ComponentShelf>
  );
};
