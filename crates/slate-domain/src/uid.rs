/// Generate a unique id as `{prefix}-{uuid_simple}` (hex, no hyphens).
pub fn uid(prefix: &str) -> String {
    format!("{}-{}", prefix, uuid::Uuid::new_v4().as_simple())
}
