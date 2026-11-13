use clap::{ArgMatches, CommandFactory, FromArgMatches, Parser};
use uuid::Uuid;

use std::io::{self, BufWriter, Write};
use std::process::ExitCode;

/// Language setting for internationalization
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Language {
    English,
    Chinese,
}

impl Language {
    /// Detect system language from environment variables
    /// Defaults to English if detection fails or language is not supported
    fn detect() -> Self {
        let locale = ["LC_ALL", "LC_MESSAGES", "LANG"]
            .into_iter()
            .filter_map(|key| std::env::var(key).ok())
            .find(|value| !value.is_empty());
        if locale.is_some_and(|value| value.to_ascii_lowercase().starts_with("zh")) {
            Self::Chinese
        } else {
            Self::English
        }
    }
}

/// Get localized messages based on language
struct Messages {
    lang: Language,
}

impl Messages {
    fn new(lang: Language) -> Self {
        Self { lang }
    }

    fn conflict_warning(&self) -> &'static str {
        match self.lang {
            Language::English => "Warning: Both -f (full) and -s (simple) format flags specified.",
            Language::Chinese => "警告：同时指定了 -f（完整）和 -s（简单）格式标志。",
        }
    }

    fn using_full(&self) -> &'static str {
        match self.lang {
            Language::English => "Using -f (full format) based on argument order.",
            Language::Chinese => "根据参数顺序使用 -f（完整格式）。",
        }
    }

    fn using_simple(&self) -> &'static str {
        match self.lang {
            Language::English => "Using -s (simple format) based on argument order.",
            Language::Chinese => "根据参数顺序使用 -s（简单格式）。",
        }
    }

    fn invalid_version(&self, version: &str) -> String {
        match self.lang {
            Language::English => format!("Invalid UUID version: {}. Valid values: 4, 7", version),
            Language::Chinese => format!("无效的 UUID 版本：{}。有效值：4、7", version),
        }
    }
}

/// UUID version to generate
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum UuidVersion {
    /// Version 4: Random UUID (default)
    #[default]
    V4,
    /// Version 7: Time-ordered UUID
    V7,
}

impl std::str::FromStr for UuidVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lang = Language::detect();
        let msgs = Messages::new(lang);

        match s {
            "4" | "v4" | "V4" => Ok(UuidVersion::V4),
            "7" | "v7" | "V7" => Ok(UuidVersion::V7),
            _ => Err(msgs.invalid_version(s)),
        }
    }
}

impl std::fmt::Display for UuidVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UuidVersion::V4 => write!(f, "v4"),
            UuidVersion::V7 => write!(f, "v7"),
        }
    }
}

/// UUID generator tool
#[derive(Parser)]
#[command(name = "zuuid", version, disable_version_flag = true)]
#[command(about = "Generate UUID v4/v7", long_about = None)]
struct Cli {
    /// Print program version (use -V to select the UUID version)
    #[arg(long = "version", action = clap::ArgAction::Version)]
    _display_version: Option<bool>,

    /// UUID version to generate (4 or 7, default: 4)
    #[arg(
        short = 'V',
        long = "uuid-version",
        visible_short_alias = 'v',
        default_value = "4"
    )]
    version: UuidVersion,

    /// Output UUID in uppercase
    #[arg(short = 'U', long = "upper", visible_short_alias = 'u')]
    uppercase: bool,

    /// Output UUID without hyphens (32 chars)
    #[arg(short = 's', long = "simple", visible_short_alias = 'S')]
    simple: bool,

    /// Output UUID with hyphens in full format (36 chars)
    #[arg(short = 'f', long = "full", visible_short_alias = 'F')]
    full: bool,

    /// Number of UUIDs to generate (default: 1)
    #[arg(short = 'n', long = "count", default_value = "1")]
    count: usize,
}

/// UUID text format, resolved once before generating any output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OutputFormat {
    Full,
    Simple,
}

/// Determine format precedence from clap's parsed occurrence indices.
/// Returns (prefer_full, conflict_detected). Defaults are not occurrences.
fn determine_format_precedence(matches: &ArgMatches) -> (bool, bool) {
    if matches.get_flag("full") && matches.get_flag("simple") {
        (matches.index_of("full") < matches.index_of("simple"), true)
    } else {
        (true, false)
    }
}

/// Print warning message in yellow
fn print_conflict_warning(prefer_full: bool) {
    let lang = Language::detect();
    let msgs = Messages::new(lang);

    let mut stderr = io::stderr().lock();
    let _ = writeln!(stderr, "\x1b[33m{}\x1b[0m", msgs.conflict_warning());
    if prefer_full {
        let _ = writeln!(stderr, "\x1b[33m{}\x1b[0m", msgs.using_full());
    } else {
        let _ = writeln!(stderr, "\x1b[33m{}\x1b[0m", msgs.using_simple());
    }
}

/// Encode UUID text in a reusable stack buffer, without per-UUID allocation.
fn encode_uuid<'a>(
    id: &Uuid,
    uppercase: bool,
    format: OutputFormat,
    buffer: &'a mut [u8; 45],
) -> &'a str {
    match (format, uppercase) {
        (OutputFormat::Full, false) => id.hyphenated().encode_lower(buffer),
        (OutputFormat::Full, true) => id.hyphenated().encode_upper(buffer),
        (OutputFormat::Simple, false) => id.simple().encode_lower(buffer),
        (OutputFormat::Simple, true) => id.simple().encode_upper(buffer),
    }
}

/// Write a batch of UUIDs, propagating output and final flush failures.
fn write_uuids(
    output: &mut impl Write,
    version: UuidVersion,
    uppercase: bool,
    format: OutputFormat,
    count: usize,
) -> io::Result<()> {
    let mut buffer = Uuid::encode_buffer();
    for _ in 0..count {
        let id = match version {
            UuidVersion::V4 => Uuid::new_v4(),
            UuidVersion::V7 => Uuid::now_v7(),
        };
        writeln!(
            output,
            "{}",
            encode_uuid(&id, uppercase, format, &mut buffer)
        )?;
    }
    output.flush()
}

fn main() -> ExitCode {
    let matches = Cli::command().get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|error| error.exit());
    let (prefer_full, conflict) = determine_format_precedence(&matches);
    if conflict {
        print_conflict_warning(prefer_full);
    }
    let format = if cli.simple && (!cli.full || !prefer_full) {
        OutputFormat::Simple
    } else {
        OutputFormat::Full
    };
    let stdout = io::stdout();
    let mut output = BufWriter::new(stdout.lock());
    match write_uuids(&mut output, cli.version, cli.uppercase, format, cli.count) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(io::stderr().lock(), "zuuid: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_uuid(version: UuidVersion, uppercase: bool, format: OutputFormat) -> String {
        let mut output = Vec::new();
        write_uuids(&mut output, version, uppercase, format, 1).unwrap();
        String::from_utf8(output)
            .unwrap()
            .trim_end_matches('\n')
            .to_owned()
    }

    #[test]
    fn test_generate_default_format() {
        let uuid = generate_uuid(UuidVersion::V4, false, OutputFormat::Full);
        // Default format: lowercase with hyphens (8-4-4-4-12)
        assert!(uuid.len() == 36);
        assert!(uuid.contains('-'));
        assert!(uuid.chars().filter(|&c| c == '-').count() == 4);
        // Check that there are no uppercase letters
        assert!(!uuid.chars().any(|c| c.is_ascii_uppercase()));
    }

    #[test]
    fn test_generate_uppercase_format() {
        let uuid = generate_uuid(UuidVersion::V4, true, OutputFormat::Full);
        // Uppercase format with hyphens
        assert!(uuid.len() == 36);
        assert!(uuid.contains('-'));
        assert!(uuid.chars().filter(|&c| c == '-').count() == 4);
        // Check that there are no lowercase letters
        assert!(!uuid.chars().any(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn test_generate_simple_format() {
        let uuid = generate_uuid(UuidVersion::V4, false, OutputFormat::Simple);
        // Simple format: lowercase without hyphens
        assert!(uuid.len() == 32);
        assert!(!uuid.contains('-'));
        // Check that there are no uppercase letters
        assert!(!uuid.chars().any(|c| c.is_ascii_uppercase()));
    }

    #[test]
    fn test_generate_uppercase_simple_format() {
        let uuid = generate_uuid(UuidVersion::V4, true, OutputFormat::Simple);
        // Uppercase simple format
        assert!(uuid.len() == 32);
        assert!(!uuid.contains('-'));
        // Check that there are no lowercase letters
        assert!(!uuid.chars().any(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn test_uuid_v4_uniqueness() {
        let uuid1 = generate_uuid(UuidVersion::V4, false, OutputFormat::Full);
        let uuid2 = generate_uuid(UuidVersion::V4, false, OutputFormat::Full);
        // Two UUIDs should be different (extremely unlikely to be the same)
        assert_ne!(uuid1, uuid2);
    }

    #[test]
    fn test_uuid_v7_uniqueness() {
        let uuid1 = generate_uuid(UuidVersion::V7, false, OutputFormat::Full);
        std::thread::sleep(std::time::Duration::from_millis(10));
        let uuid2 = generate_uuid(UuidVersion::V7, false, OutputFormat::Full);
        // Two V7 UUIDs should be different
        assert_ne!(uuid1, uuid2);
    }

    #[test]
    fn test_uuid_v7_ordered() {
        let uuid1 = generate_uuid(UuidVersion::V7, false, OutputFormat::Full);
        std::thread::sleep(std::time::Duration::from_millis(10));
        let uuid2 = generate_uuid(UuidVersion::V7, false, OutputFormat::Full);
        // V7 UUIDs should be time-ordered (uuid2 > uuid1)
        assert!(uuid2 > uuid1);
    }

    #[test]
    fn test_uuid_valid_format() {
        let uuid = generate_uuid(UuidVersion::V4, false, OutputFormat::Full);
        // Check standard UUID format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
        let parts: Vec<&str> = uuid.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
        assert_eq!(parts[4].len(), 12);
    }

    #[test]
    fn test_uuid_hex_chars() {
        let uuid = generate_uuid(UuidVersion::V4, false, OutputFormat::Simple);
        // All characters should be valid hex digits
        assert!(uuid.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_cli_parse_uppercase_short() {
        let cli = Cli::try_parse_from(["zuuid", "-U"]).unwrap();
        assert_eq!(cli.version, UuidVersion::V4);
        assert!(cli.uppercase);
        assert!(!cli.simple);
    }

    #[test]
    fn test_cli_parse_uppercase_short_lower() {
        let cli = Cli::try_parse_from(["zuuid", "-u"]).unwrap();
        assert_eq!(cli.version, UuidVersion::V4);
        assert!(cli.uppercase);
        assert!(!cli.simple);
    }

    #[test]
    fn test_cli_parse_uppercase_long() {
        let cli = Cli::try_parse_from(["zuuid", "--upper"]).unwrap();
        assert_eq!(cli.version, UuidVersion::V4);
        assert!(cli.uppercase);
        assert!(!cli.simple);
    }

    #[test]
    fn test_cli_parse_simple_short() {
        let cli = Cli::try_parse_from(["zuuid", "-s"]).unwrap();
        assert_eq!(cli.version, UuidVersion::V4);
        assert!(!cli.uppercase);
        assert!(cli.simple);
    }

    #[test]
    fn test_cli_parse_simple_short_upper() {
        let cli = Cli::try_parse_from(["zuuid", "-S"]).unwrap();
        assert_eq!(cli.version, UuidVersion::V4);
        assert!(!cli.uppercase);
        assert!(cli.simple);
    }

    #[test]
    fn test_cli_parse_simple_long() {
        let cli = Cli::try_parse_from(["zuuid", "--simple"]).unwrap();
        assert_eq!(cli.version, UuidVersion::V4);
        assert!(!cli.uppercase);
        assert!(cli.simple);
    }

    #[test]
    fn test_cli_parse_combined() {
        let cli = Cli::try_parse_from(["zuuid", "-u", "-s"]).unwrap();
        assert_eq!(cli.version, UuidVersion::V4);
        assert!(cli.uppercase);
        assert!(cli.simple);
    }

    #[test]
    fn test_cli_parse_combined_long() {
        let cli = Cli::try_parse_from(["zuuid", "--upper", "--simple"]).unwrap();
        assert_eq!(cli.version, UuidVersion::V4);
        assert!(cli.uppercase);
        assert!(cli.simple);
    }

    #[test]
    fn test_cli_parse_no_args() {
        let cli = Cli::try_parse_from(["zuuid"]).unwrap();
        assert_eq!(cli.version, UuidVersion::V4);
        assert!(!cli.uppercase);
        assert!(!cli.simple);
    }

    #[test]
    fn test_cli_parse_version_4() {
        let cli = Cli::try_parse_from(["zuuid", "-V", "4"]).unwrap();
        assert_eq!(cli.version, UuidVersion::V4);
    }

    #[test]
    fn test_cli_parse_version_7() {
        let cli = Cli::try_parse_from(["zuuid", "-V", "7"]).unwrap();
        assert_eq!(cli.version, UuidVersion::V7);
    }

    #[test]
    fn test_cli_parse_version_lowercase_v_4() {
        let cli = Cli::try_parse_from(["zuuid", "-v", "4"]).unwrap();
        assert_eq!(cli.version, UuidVersion::V4);
    }

    #[test]
    fn test_cli_parse_version_lowercase_v_7() {
        let cli = Cli::try_parse_from(["zuuid", "-v", "7"]).unwrap();
        assert_eq!(cli.version, UuidVersion::V7);
    }

    #[test]
    fn test_cli_parse_version_long() {
        let cli = Cli::try_parse_from(["zuuid", "--uuid-version", "7"]).unwrap();
        assert_eq!(cli.version, UuidVersion::V7);
    }

    #[test]
    fn test_cli_parse_version_with_other_options() {
        let cli = Cli::try_parse_from(["zuuid", "-V", "7", "-U", "-s"]).unwrap();
        assert_eq!(cli.version, UuidVersion::V7);
        assert!(cli.uppercase);
        assert!(cli.simple);
    }

    #[test]
    fn test_cli_parse_version_lowercase_v_with_other_options() {
        let cli = Cli::try_parse_from(["zuuid", "-v", "7", "-U", "-s"]).unwrap();
        assert_eq!(cli.version, UuidVersion::V7);
        assert!(cli.uppercase);
        assert!(cli.simple);
    }

    #[test]
    fn test_uuid_version_from_str() {
        assert_eq!("4".parse::<UuidVersion>().unwrap(), UuidVersion::V4);
        assert_eq!("7".parse::<UuidVersion>().unwrap(), UuidVersion::V7);
        assert_eq!("v4".parse::<UuidVersion>().unwrap(), UuidVersion::V4);
        assert_eq!("v7".parse::<UuidVersion>().unwrap(), UuidVersion::V7);
        assert_eq!("V4".parse::<UuidVersion>().unwrap(), UuidVersion::V4);
        assert_eq!("V7".parse::<UuidVersion>().unwrap(), UuidVersion::V7);
        assert!("5".parse::<UuidVersion>().is_err());
        assert!("invalid".parse::<UuidVersion>().is_err());
    }

    #[test]
    fn test_cli_parse_combined_flags_us() {
        let cli = Cli::try_parse_from(["zuuid", "-us"]).unwrap();
        assert_eq!(cli.version, UuidVersion::V4);
        assert!(cli.uppercase);
        assert!(cli.simple);
    }

    #[test]
    fn test_cli_parse_combined_flags_u_upper_s() {
        let cli = Cli::try_parse_from(["zuuid", "-uS"]).unwrap();
        assert_eq!(cli.version, UuidVersion::V4);
        assert!(cli.uppercase);
        assert!(cli.simple);
    }

    #[test]
    fn test_cli_parse_combined_flags_upper_u_s() {
        let cli = Cli::try_parse_from(["zuuid", "-Us"]).unwrap();
        assert_eq!(cli.version, UuidVersion::V4);
        assert!(cli.uppercase);
        assert!(cli.simple);
    }

    #[test]
    fn test_cli_parse_combined_flags_upper_u_upper_s() {
        let cli = Cli::try_parse_from(["zuuid", "-US"]).unwrap();
        assert_eq!(cli.version, UuidVersion::V4);
        assert!(cli.uppercase);
        assert!(cli.simple);
    }

    #[test]
    fn test_cli_parse_combined_with_version() {
        let cli = Cli::try_parse_from(["zuuid", "-V7", "-us"]).unwrap();
        assert_eq!(cli.version, UuidVersion::V7);
        assert!(cli.uppercase);
        assert!(cli.simple);
    }

    #[test]
    fn test_cli_parse_combined_all_flags() {
        let cli = Cli::try_parse_from(["zuuid", "-V7", "-U", "-s"]).unwrap();
        assert_eq!(cli.version, UuidVersion::V7);
        assert!(cli.uppercase);
        assert!(cli.simple);
    }

    #[test]
    fn test_generate_full_format() {
        let uuid = generate_uuid(UuidVersion::V4, false, OutputFormat::Full);
        // Full format: lowercase with hyphens (36 chars)
        assert!(uuid.len() == 36);
        assert!(uuid.contains('-'));
        assert!(uuid.chars().filter(|&c| c == '-').count() == 4);
    }

    #[test]
    fn test_generate_full_uppercase_format() {
        let uuid = generate_uuid(UuidVersion::V4, true, OutputFormat::Full);
        // Full uppercase format
        assert!(uuid.len() == 36);
        assert!(uuid.contains('-'));
        assert!(!uuid.chars().any(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn test_cli_parse_full_short() {
        let cli = Cli::try_parse_from(["zuuid", "-f"]).unwrap();
        assert!(cli.full);
        assert!(!cli.simple);
    }

    #[test]
    fn test_cli_parse_full_short_upper() {
        let cli = Cli::try_parse_from(["zuuid", "-F"]).unwrap();
        assert!(cli.full);
        assert!(!cli.simple);
    }

    #[test]
    fn test_cli_parse_full_long() {
        let cli = Cli::try_parse_from(["zuuid", "--full"]).unwrap();
        assert!(cli.full);
        assert!(!cli.simple);
    }

    #[test]
    fn test_cli_parse_full_with_uppercase() {
        let cli = Cli::try_parse_from(["zuuid", "-f", "-U"]).unwrap();
        assert!(cli.full);
        assert!(cli.uppercase);
    }

    #[test]
    fn test_cli_parse_combined_full_flags() {
        let cli = Cli::try_parse_from(["zuuid", "-fU"]).unwrap();
        assert!(cli.full);
        assert!(cli.uppercase);
    }

    #[test]
    fn test_cli_parse_full_uppercase_u_with_full() {
        let cli = Cli::try_parse_from(["zuuid", "-Uf"]).unwrap();
        assert!(cli.full);
        assert!(cli.uppercase);
    }

    #[test]
    fn test_cli_parse_default_count() {
        let cli = Cli::try_parse_from(["zuuid"]).unwrap();
        assert_eq!(cli.count, 1);
    }

    #[test]
    fn test_cli_parse_count_short() {
        let cli = Cli::try_parse_from(["zuuid", "-n", "5"]).unwrap();
        assert_eq!(cli.count, 5);
    }

    #[test]
    fn test_cli_parse_count_long() {
        let cli = Cli::try_parse_from(["zuuid", "--count", "10"]).unwrap();
        assert_eq!(cli.count, 10);
    }

    #[test]
    fn test_cli_parse_count_with_other_options() {
        let cli = Cli::try_parse_from(["zuuid", "-n", "3", "-U", "-s", "-V", "7"]).unwrap();
        assert_eq!(cli.count, 3);
        assert!(cli.uppercase);
        assert!(cli.simple);
        assert_eq!(cli.version, UuidVersion::V7);
    }
}
