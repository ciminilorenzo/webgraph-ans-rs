//! The module providing ANSBvGraph and ANSBvGraphSeq

use std::fmt::Display;

pub mod estimators;
pub mod factories;
pub mod random_access;
pub mod sequential;
pub mod writers;

/// An enumeration of the components composing the BVGraph format.
#[derive(Clone, Copy, Debug)]
pub enum BVGraphComponent {
    Outdegree,
    ReferenceOffset,
    BlockCount,
    Blocks,
    IntervalCount,
    IntervalStart,
    IntervalLen,
    FirstResidual,
    Residual,
}

impl BVGraphComponent {
    /// The number of components in the BVGraph format.
    pub const COMPONENTS: usize = 9;
}

impl Display for BVGraphComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BVGraphComponent::Outdegree => write!(f, "{:<15}", "Outdegree"),
            BVGraphComponent::ReferenceOffset => write!(f, "{:<15}", "ReferenceOffset"),
            BVGraphComponent::BlockCount => write!(f, "{:<15}", "BlockCount"),
            BVGraphComponent::Blocks => write!(f, "{:<15}", "Blocks"),
            BVGraphComponent::IntervalCount => write!(f, "{:<15}", "IntervalCount"),
            BVGraphComponent::IntervalStart => write!(f, "{:<15}", "IntervalStart"),
            BVGraphComponent::IntervalLen => write!(f, "{:<15}", "IntervalLen"),
            BVGraphComponent::FirstResidual => write!(f, "{:<15}", "FirstResidual"),
            BVGraphComponent::Residual => write!(f, "{:<15}", "Residual"),
        }
    }
}

impl From<usize> for BVGraphComponent {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Outdegree,
            1 => Self::ReferenceOffset,
            2 => Self::BlockCount,
            3 => Self::Blocks,
            4 => Self::IntervalCount,
            5 => Self::IntervalStart,
            6 => Self::IntervalLen,
            7 => Self::FirstResidual,
            8 => Self::Residual,
            _ => panic!("Invalid component."),
        }
    }
}
