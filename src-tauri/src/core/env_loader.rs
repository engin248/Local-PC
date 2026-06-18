use crate::core::dependency_analyzer::DependencyAnalyzer;
use std::fs;
use std::path::PathBuf;

/// Yerel `.env` / `config/secrets.env` dosyalarından API anahtarlarını yükler.
///
/// Panel, AI provider anahtarlarını (örn. `GEMINI_API_KEY`, `CURSOR_API_KEY`)
/// ortam değişkenlerinden okur. Bu yardımcı, komutanın yerel olarak kaydettiği
/// `.env` dosyasındaki değerleri süreç ortamına aktarır. Zaten ortamda tanımlı
/// olan değişkenlerin üzerine YAZMAZ; sistem/işletim ortamı her zaman önceliklidir.
pub struct EnvLoader;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct EnvLoadReport {
    pub files_read: Vec<String>,
    pub keys_loaded: usize,
    pub keys_skipped_existing: usize,
}

impl EnvLoader {
    /// Bilinen konumlardaki `.env` dosyalarını yükler ve kısa bir rapor döner.
    /// Dosya yoksa sessizce boş rapor döner (hata değildir).
    pub fn load_local_secrets() -> EnvLoadReport {
        let mut report = EnvLoadReport::default();

        for path in Self::candidate_paths() {
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            report.files_read.push(path.to_string_lossy().into_owned());
            Self::apply(&content, &mut report);
        }

        report
    }

    fn candidate_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        if let Ok(root) = DependencyAnalyzer::get_project_root() {
            paths.push(root.join(".env"));
            paths.push(root.join("config").join("secrets.env"));
        }
        paths
    }

    fn apply(content: &str, report: &mut EnvLoadReport) {
        for (key, value) in Self::parse(content) {
            match std::env::var(&key) {
                Ok(existing) if !existing.trim().is_empty() => {
                    report.keys_skipped_existing += 1;
                }
                _ => {
                    std::env::set_var(&key, value);
                    report.keys_loaded += 1;
                }
            }
        }
    }

    /// `.env` içeriğini `(anahtar, değer)` çiftlerine ayrıştırır.
    /// Desteklenen biçim: `KEY=VALUE`, `export KEY=VALUE`, `#` yorum satırları,
    /// boş satırlar ve tek/çift tırnaklı değerler.
    fn parse(content: &str) -> Vec<(String, String)> {
        let mut pairs = Vec::new();

        // Windows editörleri dosya başına BOM (\u{feff}) ekleyebilir; ilk anahtarı bozmamak için temizle.
        let content = content.strip_prefix('\u{feff}').unwrap_or(content);

        for raw_line in content.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let line = line.strip_prefix("export ").unwrap_or(line);

            let Some((key_part, value_part)) = line.split_once('=') else {
                continue;
            };

            let key = key_part.trim();
            if key.is_empty() || !Self::is_valid_key(key) {
                continue;
            }

            let value = Self::clean_value(value_part);
            pairs.push((key.to_string(), value));
        }

        pairs
    }

    fn is_valid_key(key: &str) -> bool {
        key.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
            && key.chars().next().map(|c| !c.is_ascii_digit()).unwrap_or(false)
    }

    fn clean_value(value_part: &str) -> String {
        let trimmed = value_part.trim();

        if trimmed.len() >= 2 {
            let bytes = trimmed.as_bytes();
            let first = bytes[0];
            let last = bytes[bytes.len() - 1];
            if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
                return trimmed[1..trimmed.len() - 1].to_string();
            }
        }

        // Tırnaksız değerlerde satır içi yorumları kaldır (` #` öncesi).
        if let Some(idx) = trimmed.find(" #") {
            return trimmed[..idx].trim().to_string();
        }

        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_pairs() {
        let pairs = EnvLoader::parse("GEMINI_API_KEY=abc123\nCURSOR_API_KEY=xyz789");
        assert_eq!(
            pairs,
            vec![
                ("GEMINI_API_KEY".to_string(), "abc123".to_string()),
                ("CURSOR_API_KEY".to_string(), "xyz789".to_string()),
            ]
        );
    }

    #[test]
    fn ignores_comments_blanks_and_export_prefix() {
        let content = "# yorum\n\nexport GEMINI_API_KEY=secret\n   # bir yorum daha\n";
        let pairs = EnvLoader::parse(content);
        assert_eq!(pairs, vec![("GEMINI_API_KEY".to_string(), "secret".to_string())]);
    }

    #[test]
    fn strips_quotes_and_inline_comments() {
        let pairs = EnvLoader::parse("A=\"quoted value\"\nB='single'\nC=plain # not part of value");
        assert_eq!(
            pairs,
            vec![
                ("A".to_string(), "quoted value".to_string()),
                ("B".to_string(), "single".to_string()),
                ("C".to_string(), "plain".to_string()),
            ]
        );
    }

    #[test]
    fn strips_leading_bom() {
        let pairs = EnvLoader::parse("\u{feff}GEMINI_API_KEY=withbom");
        assert_eq!(
            pairs,
            vec![("GEMINI_API_KEY".to_string(), "withbom".to_string())]
        );
    }

    #[test]
    fn rejects_invalid_keys() {
        let pairs = EnvLoader::parse("1BAD=x\nGOOD_KEY=ok\nbad-key=nope\n=novalue");
        assert_eq!(pairs, vec![("GOOD_KEY".to_string(), "ok".to_string())]);
    }

    #[test]
    fn existing_env_is_not_overwritten() {
        std::env::set_var("LOKAL_PANEL_ENV_LOADER_TEST_EXISTING", "system_value");
        let mut report = EnvLoadReport::default();
        EnvLoader::apply(
            "LOKAL_PANEL_ENV_LOADER_TEST_EXISTING=file_value\nLOKAL_PANEL_ENV_LOADER_TEST_NEW=fresh",
            &mut report,
        );
        assert_eq!(
            std::env::var("LOKAL_PANEL_ENV_LOADER_TEST_EXISTING").unwrap(),
            "system_value"
        );
        assert_eq!(
            std::env::var("LOKAL_PANEL_ENV_LOADER_TEST_NEW").unwrap(),
            "fresh"
        );
        assert_eq!(report.keys_loaded, 1);
        assert_eq!(report.keys_skipped_existing, 1);
        std::env::remove_var("LOKAL_PANEL_ENV_LOADER_TEST_EXISTING");
        std::env::remove_var("LOKAL_PANEL_ENV_LOADER_TEST_NEW");
    }
}
