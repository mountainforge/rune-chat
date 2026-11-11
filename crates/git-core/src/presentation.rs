//! Display values shared by the native frontends. All metadata comes from Git.
pub fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|s| s.chars().next())
        .take(2)
        .flat_map(char::to_uppercase)
        .collect()
}
pub fn avatar_color(name: &str) -> u32 {
    const COLORS: [u32; 6] = [0xf2c6a0, 0xa9c7ff, 0xc3a9ef, 0xb5dfc4, 0xefc0cc, 0xc7d5da];
    COLORS[name
        .bytes()
        .fold(0usize, |h, b| h.wrapping_mul(31).wrapping_add(b as usize))
        % COLORS.len()]
}
pub fn file_parts(path: &str) -> (&str, &str) {
    path.rfind('/')
        .map(|i| (&path[..=i], &path[i + 1..]))
        .unwrap_or(("", path))
}
pub fn head_label(commits: &[crate::Commit]) -> String {
    commits
        .iter()
        .flat_map(|c| c.refs.split(", "))
        .find_map(|r| r.strip_prefix("HEAD -> "))
        .unwrap_or("All branches")
        .to_owned()
}
