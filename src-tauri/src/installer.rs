// The NSIS installer writes the chosen language here (see windows/hooks.nsh).
#[cfg(windows)]
const REG_KEY: &str = r"Software\Foxivo\Sunucu Analiz";

#[cfg(windows)]
pub fn language() -> Option<String> {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    let key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(REG_KEY)
        .ok()?;
    let lang: String = key.get_value("Language").ok()?;
    matches!(lang.as_str(), "tr" | "en").then_some(lang)
}

#[cfg(not(windows))]
pub fn language() -> Option<String> {
    None
}
