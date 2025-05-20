export const preventAll = (e: MouseEvent) => {
  e.preventDefault();
  e.stopPropagation();
};
