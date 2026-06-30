export const useArrowNavigation = (
  selector = 'button:not([class*="absolute"])',
) => {
  let containerRef: HTMLDivElement | undefined;

  const onKeyDown = (e: KeyboardEvent) => {
    if (!['ArrowRight', 'ArrowLeft', 'ArrowDown', 'ArrowUp'].includes(e.key)) {
      return;
    }

    if (!containerRef) {
      return;
    }

    const elements = Array.from(
      containerRef.querySelectorAll(selector),
    ) as HTMLElement[];

    if (elements.length === 0) {
      return;
    }

    const currentIndex = elements.indexOf(
      document.activeElement as HTMLElement,
    );

    if (currentIndex === -1) {
      return;
    }

    e.preventDefault();

    let nextIndex = currentIndex;

    if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
      nextIndex = currentIndex + 1 < elements.length ? currentIndex + 1 : 0;
    } else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
      nextIndex =
        currentIndex - 1 >= 0 ? currentIndex - 1 : elements.length - 1;
    }

    elements[nextIndex]?.focus();
  };

  return {
    setRef: (el: HTMLDivElement) => {
      containerRef = el;
    },
    onKeyDown,
  };
};
