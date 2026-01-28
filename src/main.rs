use clap::Parser;
use uuid::Uuid;

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
        // Check LANG, LC_ALL, LC_MESSAGES environment variables
        for var in ["LANG", "LC_ALL", "LC_MESSAGES"] {
            if let Ok(lang) = std::env::var(var) {
                if lang.to_lowercase().starts_with("zh") {
                    return Language::Chinese;
                }
            }
        }
        // Default to English if no Chinese locale detected or on error
        Language::English
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

        match s.to_lowercase().as_str() {
            "4" | "v4" => Ok(UuidVersion::V4),
            "7" | "v7" => Ok(UuidVersion::V7),
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
#[command(name = "zuuid")]
#[command(about = "Generate UUID v4/v7", long_about = None)]
struct Cli {
    /// UUID version to generate (4 or 7, default: 4)
    #[arg(short = 'V', long = "uuid-version", visible_short_alias = 'v', default_value = "4")]
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

/// Determine format precedence based on argument order
/// Returns (prefer_full, conflict_detected)
fn determine_format_precedence() -> (bool, bool) {
    let args: Vec<String> = std::env::args().collect();

    // Find positions of format-related flags
    let mut full_pos = None;
    let mut simple_pos = None;

    for (i, arg) in args.iter().enumerate() {
        // Check for combined flags like -fs, -sf, -fS, -Sf, etc.
        if arg.starts_with('-') && arg.len() > 1 {
            let flags = &arg[1..]; // Remove leading '-'
            for (j, ch) in flags.chars().enumerate() {
                match ch {
                    'f' | 'F' => {
                        if full_pos.is_none() {
                            full_pos = Some(i * 1000 + j); // Use composite position
                        }
                    }
                    's' | 'S' => {
                        if simple_pos.is_none() {
                            simple_pos = Some(i * 1000 + j);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    match (full_pos, simple_pos) {
        (Some(f), Some(s)) => {
            // Both flags present, check order
            let prefer_full = f < s;
            (prefer_full, true)
        }
        _ => (true, false), // Default to full, no conflict
    }
}

/// Print warning message in yellow
fn print_conflict_warning(prefer_full: bool) {
    let lang = Language::detect();
    let msgs = Messages::new(lang);

    eprintln!("\x1b[33m{}\x1b[0m", msgs.conflict_warning());
    if prefer_full {
        eprintln!("\x1b[33m{}\x1b[0m", msgs.using_full());
    } else {
        eprintln!("\x1b[33m{}\x1b[0m", msgs.using_simple());
    }
}

/// Generate a formatted UUID string based on the given options
fn generate_uuid(version: UuidVersion, uppercase: bool, simple: bool, full: bool, prefer_full: bool) -> String {
    let id = match version {
        UuidVersion::V4 => Uuid::new_v4(),
        UuidVersion::V7 => Uuid::now_v7(),
    };

    // Determine format based on flags and precedence
    let output = if full && simple {
        // Both flags set, use precedence
        if prefer_full {
            id.to_string()
        } else {
            id.as_simple().to_string()
        }
    } else if full {
        id.to_string()
    } else if simple {
        id.as_simple().to_string()
    } else {
        id.to_string()
    };

    if uppercase {
        output.to_uppercase()
    } else {
        output
    }
}

fn main() {
    let (prefer_full, conflict) = determine_format_precedence();
    let cli = Cli::parse();

    if conflict {
        print_conflict_warning(prefer_full);
    }

    for _ in 0..cli.count {
        println!("{}", generate_uuid(cli.version, cli.uppercase, cli.simple, cli.full, prefer_full));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_default_format() {
        let uuid = generate_uuid(UuidVersion::V4, false, false, false, false);
        // Default format: lowercase with hyphens (8-4-4-4-12)
        assert!(uuid.len() == 36);
        assert!(uuid.contains('-'));
        assert!(uuid.chars().filter(|&c| c == '-').count() == 4);
        // Check that there are no uppercase letters
        assert!(!uuid.chars().any(|c| c.is_ascii_uppercase()));
    }

    #[test]
    fn test_generate_uppercase_format() {
        let uuid = generate_uuid(UuidVersion::V4, true, false, false, false);
        // Uppercase format with hyphens
        assert!(uuid.len() == 36);
        assert!(uuid.contains('-'));
        assert!(uuid.chars().filter(|&c| c == '-').count() == 4);
        // Check that there are no lowercase letters
        assert!(!uuid.chars().any(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn test_generate_simple_format() {
        let uuid = generate_uuid(UuidVersion::V4, false, true, false, false);
        // Simple format: lowercase without hyphens
        assert!(uuid.len() == 32);
        assert!(!uuid.contains('-'));
        // Check that there are no uppercase letters
        assert!(!uuid.chars().any(|c| c.is_ascii_uppercase()));
    }

    #[test]
    fn test_generate_uppercase_simple_format() {
        let uuid = generate_uuid(UuidVersion::V4, true, true, false, false);
        // Uppercase simple format
        assert!(uuid.len() == 32);
        assert!(!uuid.contains('-'));
        // Check that there are no lowercase letters
        assert!(!uuid.chars().any(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn test_uuid_v4_uniqueness() {
        let uuid1 = generate_uuid(UuidVersion::V4, false, false, false, false);
        let uuid2 = generate_uuid(UuidVersion::V4, false, false, false, false);
        // Two UUIDs should be different (extremely unlikely to be the same)
        assert_ne!(uuid1, uuid2);
    }

    #[test]
    fn test_uuid_v7_uniqueness() {
        let uuid1 = generate_uuid(UuidVersion::V7, false, false, false, false);
        std::thread::sleep(std::time::Duration::from_millis(10));
        let uuid2 = generate_uuid(UuidVersion::V7, false, false, false, false);
        // Two V7 UUIDs should be different
        assert_ne!(uuid1, uuid2);
    }

    #[test]
    fn test_uuid_v7_ordered() {
        let uuid1 = generate_uuid(UuidVersion::V7, false, false, false, false);
        std::thread::sleep(std::time::Duration::from_millis(10));
        let uuid2 = generate_uuid(UuidVersion::V7, false, false, false, false);
        // V7 UUIDs should be time-ordered (uuid2 > uuid1)
        assert!(uuid2 > uuid1);
    }

    #[test]
    fn test_uuid_valid_format() {
        let uuid = generate_uuid(UuidVersion::V4, false, false, false, false);
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
        let uuid = generate_uuid(UuidVersion::V4, false, true, false, false);
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
        let uuid = generate_uuid(UuidVersion::V4, false, false, true, true);
        // Full format: lowercase with hyphens (36 chars)
        assert!(uuid.len() == 36);
        assert!(uuid.contains('-'));
        assert!(uuid.chars().filter(|&c| c == '-').count() == 4);
    }

    #[test]
    fn test_generate_full_uppercase_format() {
        let uuid = generate_uuid(UuidVersion::V4, true, false, true, true);
        // Full uppercase format
        assert!(uuid.len() == 36);
        assert!(uuid.contains('-'));
        assert!(!uuid.chars().any(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn test_conflict_simple_wins() {
        let uuid = generate_uuid(UuidVersion::V4, false, true, true, false);
        // When prefer_full is false, simple wins
        assert!(uuid.len() == 32);
        assert!(!uuid.contains('-'));
    }

    #[test]
    fn test_conflict_full_wins() {
        let uuid = generate_uuid(UuidVersion::V4, false, true, true, true);
        // When prefer_full is true, full wins
        assert!(uuid.len() == 36);
        assert!(uuid.contains('-'));
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
