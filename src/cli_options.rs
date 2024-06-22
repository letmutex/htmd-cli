use crate::config_util::read_cli_options_from_toml_file;
use clap::{Arg, ArgAction, ArgMatches};
use htmd::options::{
    BrStyle, BulletListMarker, CodeBlockFence, CodeBlockStyle, HeadingStyle, HrStyle,
    LinkReferenceStyle, LinkStyle, Options,
};

pub(crate) struct CliOptions {
    pub converter_options: Options,
    pub ignored_tags: Option<Vec<String>>,
    pub flatten_output: bool,
}

pub(crate) fn parse_cli_options(matches: &ArgMatches) -> CliOptions {
    if let Some(config) = matches.get_one::<String>("options-file") {
        read_cli_options_from_toml_file(config).expect("Failed to parse options from config file")
    } else {
        CliOptions {
            converter_options: parse_converter_options_from_cli_args(matches),
            ignored_tags: parse_ignored_tags(matches),
            flatten_output: *matches.get_one::<bool>("flatten-output").unwrap(),
        }
    }
}

fn parse_ignored_tags(matches: &ArgMatches) -> Option<Vec<String>> {
    let Some(tags_str) = matches.get_one::<String>("ignored-tags") else {
        return None;
    };
    Some(tags_str.split(",").map(|tag| tag.to_string()).collect())
}

fn parse_converter_options_from_cli_args(matches: &ArgMatches) -> Options {
    let heading_style = match matches.get_one::<String>("heading-style").unwrap().as_str() {
        "setex" => HeadingStyle::Setex,
        "atx" | _ => HeadingStyle::Atx,
    };

    let hr_style = match matches.get_one::<String>("hr-style").unwrap().as_str() {
        "dashes" => HrStyle::Dashes,
        "underscores" => HrStyle::Underscores,
        "asterisks" | _ => HrStyle::Asterisks,
    };

    let br_style = match matches.get_one::<String>("br-style").unwrap().as_str() {
        "backslash" => BrStyle::Backslash,
        "two-spaces" | _ => BrStyle::TwoSpaces,
    };

    let link_style = match matches.get_one::<String>("link-style").unwrap().as_str() {
        "referenced" => LinkStyle::Referenced,
        "inlined" | _ => LinkStyle::Inlined,
    };

    let link_reference_style = match matches
        .get_one::<String>("link-reference-style")
        .unwrap()
        .as_str()
    {
        "collapsed" => LinkReferenceStyle::Collapsed,
        "shortcut" => LinkReferenceStyle::Shortcut,
        "full" | _ => LinkReferenceStyle::Full,
    };

    let code_block_style = match matches
        .get_one::<String>("code-block-style")
        .unwrap()
        .as_str()
    {
        "indented" => CodeBlockStyle::Indented,
        "fenced" | _ => CodeBlockStyle::Fenced,
    };

    let code_block_fence = match matches
        .get_one::<String>("code-block-fence")
        .unwrap()
        .as_str()
    {
        "tildes" => CodeBlockFence::Tildes,
        "backticks" | _ => CodeBlockFence::Backticks,
    };

    let bullet_list_marker = match matches
        .get_one::<String>("bullet-list-marker")
        .unwrap()
        .as_str()
    {
        "dash" => BulletListMarker::Dash,
        "asterisk" | _ => BulletListMarker::Asterisk,
    };

    let preformatted_code = *matches.get_one::<bool>("preformatted-code").unwrap();

    Options {
        heading_style,
        hr_style,
        br_style,
        link_style,
        link_reference_style,
        code_block_style,
        code_block_fence,
        bullet_list_marker,
        preformatted_code,
    }
}

pub(crate) fn cli_args() -> Vec<Arg> {
    vec![
        Arg::new("input-unnamed").index(1).num_args(1),
        Arg::new("input")
            .short('i')
            .long("input")
            .help("Specify input. Can be stdin ('-'), file, directory, or glob pattern; defaults to stdin")
            .num_args(1),
        Arg::new("output")
            .short('o')
            .long("output")
            .help(
                "Specify output. Can be stdout ('-'), file, or directory; defaults to stdout",
            )
            .num_args(1),
        Arg::new("options-file")
            .long("options-file")
            .help(
                "Read cli options from a toml file. Options are within [options] section;\n\
                if specified, other options will be ignored except for input and output",
            )
            .num_args(1),
        Arg::new("flatten-output")
            .long("flatten-output")
            .help("Flat the output files in the output folder")
            .action(ArgAction::SetTrue),
        Arg::new("ignored-tags")
            .long("ignored-tags")
            .help("Set an HTML tag list to be ignored, separated by commas")
            .num_args(1),
        Arg::new("heading-style")
            .long("heading-style")
            .num_args(1)
            .default_value("atx")
            .default_missing_value("atx")
            .value_parser(["atx", "setex"]),
        Arg::new("hr-style")
            .long("hr-style")
            .num_args(1)
            .default_value("asterisks")
            .default_missing_value("asterisks")
            .value_parser(["dashes", "asterisks", "underscores"]),
        Arg::new("br-style")
            .long("br-style")
            .num_args(1)
            .default_value("two-spaces")
            .default_missing_value("two-spaces")
            .value_parser(["two-spaces", "backslash"]),
        Arg::new("link-style")
            .long("link-style")
            .num_args(1)
            .default_value("inlined")
            .default_missing_value("inlined")
            .value_parser(["inlined", "referenced"]),
        Arg::new("link-reference-style")
            .long("link-reference-style")
            .num_args(1)
            .default_value("full")
            .default_missing_value("full")
            .value_parser(["full", "collapsed", "shortcut"]),
        Arg::new("code-block-style")
            .long("code-block-style")
            .num_args(1)
            .default_value("fenced")
            .default_missing_value("fenced")
            .value_parser(["fenced", "intended"]),
        Arg::new("code-block-fence")
            .long("code-block-fence")
            .num_args(1)
            .default_value("backticks")
            .default_missing_value("backticks")
            .value_parser(["tildes", "backticks"]),
        Arg::new("bullet-list-marker")
            .long("bullet-list-marker")
            .num_args(1)
            .default_value("asterisk")
            .default_missing_value("asterisk")
            .value_parser(["dash", "asterisk"]),
        Arg::new("preformatted-code")
            .long("preformatted-code")
            .help("Preserve whitespace in inline code tags")
            .action(ArgAction::SetTrue),
        Arg::new("version")
            .short('v')
            .long("version")
            .help("Print version")
            .num_args(0),
    ]
}
