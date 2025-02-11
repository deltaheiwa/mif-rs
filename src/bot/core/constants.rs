pub const DEFAULT_PREFIX: &str = "m.";
pub const DEFAULT_LANGUAGE: &str = "en";
// Only used in utils/apicallers/wolvesville but separated for clarity. Shouldn't be changed unless the API changes (https://api-docs.wolvesville.com).
pub const WOLVESVILLE_API_URL: &str = "https://api.wolvesville.com";

#[allow(dead_code)]
pub mod embed_limits {
    pub const EMBED_TITLE_LIMIT: usize = 256;
    pub const EMBED_DESCRIPTION_LIMIT: usize = 4096;
    pub const EMBED_FIELD_AMOUNT_LIMIT: usize = 25;
    pub const EMBED_FIELD_NAME_LIMIT: usize = 256;
    pub const EMBED_FIELD_VALUE_LIMIT: usize = 1024;
    pub const EMBED_FOOTER_LIMIT: usize = 2048;
    pub const EMBED_AUTHOR_NAME_LIMIT: usize = 256;
    pub const EMBED_TOTAL_CHARACTERS_LIMIT: usize = 6000;
}