import type { RouteSectionProps } from '@solidjs/router';

import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { cn } from '../../../shared/lib';
import { BadgeLab } from './BadgeLab';
import { ButtonsLab } from './ButtonsLab';
import { CheckboxLab } from './CheckboxLab';
import { CollapsibleLab } from './CollapsibleLab';
import { ContextMenuLab } from './ContextMenuLab';
import { DialogLab } from './DialogLab';
import { DropdownMenuLab } from './DropdownMenuLab';
import { PaginationLab } from './PaginationLab';
import { PopoverLab } from './PopoverLab';
import { ProgressCircleLab } from './ProgressCircleLab';
import { ProgressLab } from './ProgressLab';
import { SelectLab } from './SelectsLab';
import { SeparatorLab } from './SeparatorLab';
import { SkeletonLab } from './SkeletonLab';
import { SliderLab } from './SliderLab';
import { SwitchLab } from './SwitchLab';
import { TabsLab } from './TabsLab';
import { TextFieldLab } from './TextFieldLab';
import { ToastLab } from './ToastLab';
import { ToggleGroupLab } from './ToggleGroupLab';
import { TooltipLab } from './TooltipLab';

export type PlaygroundPageProps = ComponentProps<'div'> & RouteSectionProps;

export const PlaygroundPage: Component<PlaygroundPageProps> = (props) => {
  const [local, others] = splitProps(props, [
    'class',
    'params',
    'location',
    'data',
  ]);

  return (
    <div
      class={cn(
        'p-page size-full flex grow flex-col text-foreground',
        local.class,
      )}
      {...others}
    >
      <div class='flex grow flex-col overflow-auto'>
        <BadgeLab />
        <ButtonsLab />
        <CheckboxLab />
        <CollapsibleLab />
        <ContextMenuLab />
        <DialogLab />
        <DropdownMenuLab />
        <PaginationLab />
        <PopoverLab />
        <ProgressLab />
        <ProgressCircleLab />
        <SelectLab />
        <SeparatorLab />
        <SkeletonLab />
        <SliderLab />
        <SwitchLab />
        <TabsLab />
        <TextFieldLab />
        <ToastLab />
        <ToggleGroupLab />
        <TooltipLab />
      </div>
    </div>
  );
};
