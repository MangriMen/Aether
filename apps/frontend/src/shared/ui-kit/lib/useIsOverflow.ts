import { createSignal, onCleanup } from 'solid-js';

export function useIsOverflow() {
  const [isOverflowed, setIsOverflowed] = createSignal(false);
  let elementRef: HTMLElement | undefined;

  // Just save the element reference without attaching listeners here
  // Solid passes undefined to ref callbacks on unmount.
  const ref = (el: HTMLElement | undefined) => {
    elementRef = el;
  };

  onCleanup(() => {
    elementRef = undefined;
  });

  // Run the check manually when needed (e.g., on parent hover)
  const updateOverflow = () => {
    if (elementRef) {
      setIsOverflowed(elementRef.scrollWidth > elementRef.clientWidth);
    }
  };

  return [isOverflowed, ref, updateOverflow] as const;
}
