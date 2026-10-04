use eva::data;

use crate::types::mark;

#[data(error, display("no such badge: {badge}"))]
pub struct NoSuchBadge {
    pub badge: mark::Badge,
}
