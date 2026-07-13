/** 轻量列表窗口：固定行高 + overscan，无第三方依赖。 */

export type VirtualWindow = {
  start: number;
  end: number;
  offsetTop: number;
  totalHeight: number;
};

export function computeVirtualWindow(
  scrollTop: number,
  viewportHeight: number,
  itemCount: number,
  itemHeight: number,
  overscan = 4,
): VirtualWindow {
  const totalHeight = Math.max(0, itemCount * itemHeight);
  if (itemCount === 0 || itemHeight <= 0) {
    return { start: 0, end: 0, offsetTop: 0, totalHeight: 0 };
  }
  const rawStart = Math.floor(scrollTop / itemHeight);
  const visible = Math.ceil(viewportHeight / itemHeight) + 1;
  const start = Math.max(0, rawStart - overscan);
  const end = Math.min(itemCount, rawStart + visible + overscan);
  return {
    start,
    end,
    offsetTop: start * itemHeight,
    totalHeight,
  };
}
