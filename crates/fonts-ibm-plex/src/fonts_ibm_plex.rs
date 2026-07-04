use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};
use rust_embed::RustEmbed;

/// Embedded IBM Plex Sans + IBM Plex Mono font assets (SIL OFL 1.1).
///
/// Register at app startup with `Application::with_assets(fonts_ibm_plex::Assets)`
/// (or a multiplexing `AssetSource` combining this with other asset crates) so
/// that the embedded `.ttf` files resolve via `font_kit::Loader::from_bytes()`.
#[derive(RustEmbed)]
#[folder = "assets"]
#[include = "fonts/**/*.ttf"]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(Self::get(path).map(|f| f.data))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter(|p| p.starts_with(path))
            .map(|p| SharedString::from(p.to_string()))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embeds_all_seven_ibm_plex_files() {
        let expected = [
            "fonts/IBMPlexSans-Regular.ttf",
            "fonts/IBMPlexSans-Medium.ttf",
            "fonts/IBMPlexSans-SemiBold.ttf",
            "fonts/IBMPlexSans-Bold.ttf",
            "fonts/IBMPlexMono-Regular.ttf",
            "fonts/IBMPlexMono-Medium.ttf",
            "fonts/IBMPlexMono-SemiBold.ttf",
        ];

        for path in expected {
            assert!(Assets::get(path).is_some(), "missing embedded font: {path}");
        }
    }
}
