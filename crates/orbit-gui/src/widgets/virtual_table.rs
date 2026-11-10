#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtualSlice {
    pub start_index: usize,
    pub end_index: usize,
    pub top_spacer_height: f32,
    pub bottom_spacer_height: f32,
}

pub fn compute_virtual_slice(
    total_items: usize,
    row_height: f32,
    scroll_offset_y: f32,
    viewport_height: f32,
    overscan: usize,
) -> VirtualSlice {
    if total_items == 0 || row_height <= 0.0 {
        return VirtualSlice {
            start_index: 0,
            end_index: 0,
            top_spacer_height: 0.0,
            bottom_spacer_height: 0.0,
        };
    }

    let offset = scroll_offset_y.max(0.0);
    let first_visible = (offset / row_height).floor() as usize;
    let visible_count = (viewport_height / row_height).ceil() as usize;

    let start_index = first_visible.saturating_sub(overscan).min(total_items);
    let end_index = (first_visible + visible_count + overscan).min(total_items);

    let top_spacer_height = start_index as f32 * row_height;
    let bottom_spacer_height = (total_items - end_index) as f32 * row_height;

    VirtualSlice {
        start_index,
        end_index,
        top_spacer_height,
        bottom_spacer_height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_virtual_slice_10k_rows_bounds_and_spacers() {
        let slice = compute_virtual_slice(10_000, 32.0, 3200.0, 640.0, 5);

        assert_eq!(slice.start_index, 95);
        assert_eq!(slice.end_index, 125);
        assert_eq!(slice.top_spacer_height, 3040.0);
        assert_eq!(slice.bottom_spacer_height, (10_000.0 - 125.0) * 32.0);
    }

    #[test]
    fn test_compute_virtual_slice_empty_and_small_lists() {
        let empty = compute_virtual_slice(0, 32.0, 0.0, 640.0, 5);
        assert_eq!(empty.start_index, 0);
        assert_eq!(empty.end_index, 0);
        assert_eq!(empty.top_spacer_height, 0.0);
        assert_eq!(empty.bottom_spacer_height, 0.0);

        let small = compute_virtual_slice(5, 32.0, 0.0, 640.0, 5);
        assert_eq!(small.start_index, 0);
        assert_eq!(small.end_index, 5);
        assert_eq!(small.top_spacer_height, 0.0);
        assert_eq!(small.bottom_spacer_height, 0.0);
    }
}
