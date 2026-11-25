//! Hand-written CLI argument parser (contracts/native-cli.md).

use proto123_theme::{GlyphSet, ThemeId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CliAction {
    Run {
        theme: Option<ThemeId>,
        glyphs: Option<GlyphSet>,
    },
    Help,
    Version,
}

pub const USAGE: &str = "\
Usage: proto123-term [OPTIONS]

Options:
  --theme <high-contrast|console16|mono>  Override detected color theme
  --glyphs <box|ascii>                    Override detected border glyph set
  -h, --help                              Show this help message
  -V, --version                           Show version";

pub fn parse_args<'a>(args: impl IntoIterator<Item = &'a str>) -> Result<CliAction, String> {
    let mut theme: Option<ThemeId> = None;
    let mut glyphs: Option<GlyphSet> = None;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg {
            "-h" | "--help" => return Ok(CliAction::Help),
            "-V" | "--version" => return Ok(CliAction::Version),
            "--theme" => {
                let val = iter.next().ok_or_else(|| {
                    "missing value for --theme (expected high-contrast, console16, or mono)"
                        .to_owned()
                })?;
                theme = Some(ThemeId::parse(val).ok_or_else(|| {
                    format!("unknown --theme {val:?} (expected high-contrast, console16, or mono)")
                })?);
            }
            "--glyphs" => {
                let val = iter.next().ok_or_else(|| {
                    "missing value for --glyphs (expected box or ascii)".to_owned()
                })?;
                glyphs =
                    Some(GlyphSet::parse(val).ok_or_else(|| {
                        format!("unknown --glyphs {val:?} (expected box or ascii)")
                    })?);
            }
            other if other.starts_with("--theme=") => {
                let val = other.strip_prefix("--theme=").unwrap_or("");
                theme = Some(ThemeId::parse(val).ok_or_else(|| {
                    format!("unknown --theme {val:?} (expected high-contrast, console16, or mono)")
                })?);
            }
            other if other.starts_with("--glyphs=") => {
                let val = other.strip_prefix("--glyphs=").unwrap_or("");
                glyphs =
                    Some(GlyphSet::parse(val).ok_or_else(|| {
                        format!("unknown --glyphs {val:?} (expected box or ascii)")
                    })?);
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown option {other:?}\n{USAGE}"));
            }
            positional => {
                return Err(format!(
                    "unexpected positional argument {positional:?}: file and URL loading is not supported in this version"
                ));
            }
        }
    }
    Ok(CliAction::Run { theme, glyphs })
}
