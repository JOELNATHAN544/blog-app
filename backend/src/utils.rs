use slug::slugify;

/// Generate a URL-friendly slug from a title
pub fn generate_slug(title: &str) -> String {
    slugify(title)
}

// Other utility functions will be added when we implement post creation/editing
