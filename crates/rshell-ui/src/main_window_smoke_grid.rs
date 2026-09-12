use rshell_core::{PaneId, PaneTree, SplitAxis};

pub(super) fn is_grid(tree: &PaneTree) -> bool {
    let PaneTree::Split {
        axis,
        first,
        second,
        ..
    } = tree
    else {
        return false;
    };
    leaf_pair(first, orthogonal(*axis)) && leaf_pair(second, orthogonal(*axis))
}

pub(super) fn next_grid_split(tree: &PaneTree) -> Option<(PaneId, SplitAxis)> {
    match tree {
        PaneTree::Leaf { pane_id, .. } => Some((*pane_id, SplitAxis::Horizontal)),
        PaneTree::Split {
            axis,
            first,
            second,
            ..
        } => {
            let cross = orthogonal(*axis);
            match (first.as_ref(), second.as_ref()) {
                (PaneTree::Leaf { pane_id, .. }, PaneTree::Leaf { .. }) => Some((*pane_id, cross)),
                (PaneTree::Leaf { pane_id, .. }, other) if leaf_pair(other, cross) => {
                    Some((*pane_id, cross))
                }
                (other, PaneTree::Leaf { pane_id, .. }) if leaf_pair(other, cross) => {
                    Some((*pane_id, cross))
                }
                _ => None,
            }
        }
    }
}

fn leaf_pair(tree: &PaneTree, expected: SplitAxis) -> bool {
    matches!(tree, PaneTree::Split { axis, first, second, .. }
        if *axis == expected && matches!(first.as_ref(), PaneTree::Leaf { .. })
            && matches!(second.as_ref(), PaneTree::Leaf { .. }))
}

fn orthogonal(axis: SplitAxis) -> SplitAxis {
    match axis {
        SplitAxis::Horizontal => SplitAxis::Vertical,
        SplitAxis::Vertical => SplitAxis::Horizontal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split(tree: PaneTree, pane: PaneId, axis: SplitAxis) -> PaneTree {
        tree.split(pane, axis, PaneId::new(), 0.5).unwrap()
    }

    #[test]
    fn grid_requires_two_orthogonal_leaf_pairs_in_either_orientation() {
        for axis in [SplitAxis::Horizontal, SplitAxis::Vertical] {
            let pane = PaneId::new();
            let leaf = PaneTree::leaf(pane);
            assert!(!is_grid(&leaf));
            assert_eq!(next_grid_split(&leaf), Some((pane, SplitAxis::Horizontal)));
            let two = split(leaf, pane, axis);
            assert!(!is_grid(&two));
            assert_eq!(next_grid_split(&two), Some((pane, orthogonal(axis))));
            let other = two.pane_ids()[1];
            for half in [pane, other] {
                let three = split(two.clone(), half, orthogonal(axis));
                let remaining = if half == pane { other } else { pane };
                assert!(!is_grid(&three));
                assert_eq!(next_grid_split(&three), Some((remaining, orthogonal(axis))));
                let grid = split(three.clone(), remaining, orthogonal(axis));
                assert!(is_grid(&grid));
                assert_eq!(next_grid_split(&grid), None);
                let comb = split(three, half, axis);
                assert!(!is_grid(&comb));
                assert_eq!(next_grid_split(&comb), None);
                let five = split(grid, half, axis);
                assert!(!is_grid(&five));
                assert_eq!(next_grid_split(&five), None);
            }
            let parallel_three = split(two, pane, axis);
            assert!(!is_grid(&parallel_three));
            assert_eq!(next_grid_split(&parallel_three), None);
        }
    }

    #[test]
    fn planned_commands_build_a_real_grid_without_active_pane_assumptions() {
        let mut tree = PaneTree::leaf(PaneId::new());
        for _ in 0..3 {
            let (pane, axis) = next_grid_split(&tree).unwrap();
            tree = split(tree, pane, axis);
        }
        assert!(is_grid(&tree));
        assert_eq!(next_grid_split(&tree), None);
    }

    #[test]
    fn malformed_four_leaf_comb_can_reduce_then_rebuild() {
        let pane = PaneId::new();
        let mut tree = PaneTree::leaf(pane);
        for axis in [
            SplitAxis::Horizontal,
            SplitAxis::Vertical,
            SplitAxis::Horizontal,
        ] {
            let last = *tree.pane_ids().last().unwrap();
            tree = split(tree, last, axis);
        }
        assert_eq!(tree.pane_ids().len(), 4);
        assert!(!is_grid(&tree));
        assert_eq!(next_grid_split(&tree), None);
        for _ in 0..8 {
            if is_grid(&tree) {
                return;
            }
            tree = if let Some((pane, axis)) = next_grid_split(&tree) {
                split(tree, pane, axis)
            } else {
                let pane = *tree.pane_ids().last().unwrap();
                tree.close(pane).unwrap()
            };
        }
        panic!("malformed shape must converge without count-four readiness or a stall");
    }
}
