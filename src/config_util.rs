use std::{error::Error, fmt, fs};

use htmd::options::{
    BrStyle, BulletListMarker, CodeBlockFence, CodeBlockStyle, HeadingStyle, HrStyle,
    LinkReferenceStyle, LinkStyle, Options,
};
use toml::Value;

use crate::cli_options::CliOptions;

#[derive(Debug)]
pub(crate) struct ParseConfigError {
    pub message: String,
}

impl ParseConfigError {
    fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for ParseConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ParseConfigError: {}", self.message)
    }
}

impl Error for ParseConfigError {}

pub(crate) fn read_cli_options_from_toml_file(
    filepath: &str,
) -> Result<CliOptions, Box<dyn Error>> {
    let text = fs::read_to_string(filepath)?;
    let value: Value = toml::from_str(&text)?;

    let Some(options) = value.get("options") else {
        return Err(parse_config_err("No [options] in the config file."));
    };

    let converter_options = read_converter_options(options)?;
    let ignored_tags = read_ignored_tags(options)?;
    let flatten_output = options
        .get("flatten-output")
        .map(|value| value.as_bool().unwrap_or(false))
        .unwrap_or(false);
    let scripting_enabled = options
        .get("scripting-enabled")
        .map(|value| value.as_bool().unwrap_or(true))
        .unwrap_or(false);

    let options = CliOptions {
        converter_options,
        ignored_tags,
        flatten_output,
        scripting_enabled,
    };

    Ok(options)
}

fn read_converter_options(options: &Value) -> Result<Options, Box<dyn Error>> {
    let default_options = Options::default();

    let heading_style = map_options_str_field(options, "heading-style", |value| match value {
        None => Ok(default_options.heading_style),
        Some("atx") => Ok(HeadingStyle::Atx),
        Some("setex") => Ok(HeadingStyle::Setex),
        _ => Err(parse_config_err(format!(
            "Unknown heading-style value '{:?}'",
            value.unwrap()
        ))),
    })?;

    let hr_style = map_options_str_field(options, "hr-style", |value| match value {
        None => Ok(default_options.hr_style),
        Some("asterisks") => Ok(HrStyle::Asterisks),
        Some("dashes") => Ok(HrStyle::Dashes),
        Some("underscores") => Ok(HrStyle::Underscores),
        _ => Err(parse_config_err(format!(
            "Unknown hr-style value '{:?}'",
            value.unwrap()
        ))),
    })?;

    let br_style = map_options_str_field(options, "br-style", |value| match value {
        None => Ok(default_options.br_style),
        Some("two-spaces") => Ok(BrStyle::TwoSpaces),
        Some("backslash") => Ok(BrStyle::Backslash),
        _ => Err(parse_config_err(format!(
            "Unknown br-style value '{:?}'",
            value.unwrap()
        ))),
    })?;

    let link_style = map_options_str_field(options, "link-style", |value| match value {
        None => Ok(default_options.link_style),
        Some("inlined") => Ok(LinkStyle::Inlined),
        Some("inlined-prefer-autolinks") => Ok(LinkStyle::InlinedPreferAutolinks),
        Some("referenced") => Ok(LinkStyle::Referenced),
        _ => Err(parse_config_err(format!(
            "Unknown link-style value '{:?}'",
            value.unwrap()
        ))),
    })?;

    let link_reference_style =
        map_options_str_field(options, "link-reference-style", |value| match value {
            None => Ok(default_options.link_reference_style),
            Some("full") => Ok(LinkReferenceStyle::Full),
            Some("collapsed") => Ok(LinkReferenceStyle::Collapsed),
            Some("shortcut") => Ok(LinkReferenceStyle::Shortcut),
            _ => Err(parse_config_err(format!(
                "Unknown link-reference-style value '{:?}'",
                value.unwrap()
            ))),
        })?;

    let code_block_style =
        map_options_str_field(options, "code-block-style", |value| match value {
            None => Ok(default_options.code_block_style),
            Some("fenced") => Ok(CodeBlockStyle::Fenced),
            Some("indented") => Ok(CodeBlockStyle::Indented),
            _ => Err(parse_config_err(format!(
                "Unknown code-block-style value '{:?}'",
                value.unwrap()
            ))),
        })?;

    let code_block_fence =
        map_options_str_field(options, "code-block-fence", |value| match value {
            None => Ok(default_options.code_block_fence),
            Some("backticks") => Ok(CodeBlockFence::Backticks),
            Some("tildes") => Ok(CodeBlockFence::Tildes),
            _ => Err(parse_config_err(format!(
                "Unknown code-block-fence value '{:?}'",
                value.unwrap()
            ))),
        })?;

    let bullet_list_marker =
        map_options_str_field(options, "bullet-list-marker", |value| match value {
            None => Ok(default_options.bullet_list_marker),
            Some("asterisk") => Ok(BulletListMarker::Asterisk),
            Some("dash") => Ok(BulletListMarker::Dash),
            _ => Err(parse_config_err(format!(
                "Unknown bullet-list-marker value '{:?}'",
                value.unwrap()
            ))),
        })?;

    let preformatted_code =
        map_options_bool_field(options, "preformatted-code", |value| match value {
            None => Ok(default_options.preformatted_code),
            Some(false) => Ok(false),
            Some(true) => Ok(true),
        })?;

    let ul_bullet_spacing =
        map_options_u8_field(options, "ul-bullet-spacing", |value| match value {
            None => Ok(default_options.ul_bullet_spacing),
            Some(val) => Ok(val),
        })?;

    let ol_number_spacing =
        map_options_u8_field(options, "ol-number-spacing", |value| match value {
            None => Ok(default_options.ol_number_spacing),
            Some(val) => Ok(val),
        })?;

    let options = Options {
        heading_style,
        hr_style,
        br_style,
        link_style,
        link_reference_style,
        code_block_style,
        code_block_fence,
        bullet_list_marker,
        ul_bullet_spacing,
        ol_number_spacing,
        preformatted_code,
    };

    Ok(options)
}

fn read_ignored_tags(options: &Value) -> Result<Option<Vec<String>>, Box<dyn Error>> {
    let Some(value) = options.get("ignored-tags") else {
        return Ok(None);
    };
    let Some(array) = value.as_array() else {
        return Err(parse_config_err(
            "options.ignored-tags must be an string array",
        ));
    };
    let mut tags: Vec<String> = vec![];
    for tag in array {
        let Some(tag) = tag.as_str() else {
            return Err(parse_config_err(
                "Non-string element found in options.ignored-tags",
            ));
        };
        tags.push(tag.to_string());
    }
    Ok(Some(tags))
}

fn map_options_str_field<F, R>(options: &Value, name: &str, map_fn: F) -> Result<R, Box<dyn Error>>
where
    F: FnOnce(Option<&str>) -> Result<R, Box<dyn Error>>,
{
    if let Some(value) = options.get(name) {
        let Some(value) = value.as_str() else {
            return Err(parse_config_err(format!(
                "options.{} must be a string",
                name
            )));
        };
        map_fn(Some(value))
    } else {
        map_fn(None)
    }
}

fn map_options_bool_field<F, R>(options: &Value, name: &str, map_fn: F) -> Result<R, Box<dyn Error>>
where
    F: FnOnce(Option<bool>) -> Result<R, Box<dyn Error>>,
{
    if let Some(value) = options.get(name) {
        let Some(value) = value.as_bool() else {
            return Err(parse_config_err(format!(
                "options.{} must be a boolean",
                name
            )));
        };
        map_fn(Some(value))
    } else {
        map_fn(None)
    }
}

fn map_options_u8_field<F, R>(options: &Value, name: &str, map_fn: F) -> Result<R, Box<dyn Error>>
where
    F: FnOnce(Option<u8>) -> Result<R, Box<dyn Error>>,
{
    if let Some(value) = options.get(name) {
        let Some(value) = value.as_integer() else {
            return Err(parse_config_err(format!(
                "options.{} must be an u8 integer",
                name
            )));
        };
        map_fn(Some(value.try_into().unwrap()))
    } else {
        map_fn(None)
    }
}

fn parse_config_err<S>(message: S) -> Box<ParseConfigError>
where
    S: AsRef<str>,
{
    Box::new(ParseConfigError::new(message.as_ref()))
}
