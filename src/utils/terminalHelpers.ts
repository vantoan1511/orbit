export interface TerminalDimensions {
  cols: number
  rows: number
}

/**
 * Determines whether a terminal resize event should be dispatched.
 * Returns true only if dimensions are valid (> 0) and differ from the last dispatched dimensions.
 */
export function shouldDispatchTerminalResize(
  newDims: TerminalDimensions | null | undefined,
  lastDims: TerminalDimensions | null | undefined
): boolean {
  if (!newDims || newDims.cols <= 0 || newDims.rows <= 0) return false
  if (!lastDims) return true
  return newDims.cols !== lastDims.cols || newDims.rows !== lastDims.rows
}

/**
 * Returns the CSS transition class for the terminal panel.
 * Disables transitions during live mouse dragging to ensure 1:1 responsive dragging,
 * while providing smooth transition for programmatically toggled states (e.g. maximize).
 */
export function getTerminalPanelTransitionClass(isDragging: boolean): string {
  return isDragging ? 'transition-none' : 'transition-[height] duration-150 ease-out'
}
