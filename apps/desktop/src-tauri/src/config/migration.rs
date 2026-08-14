use tracing::warn;

use super::DropVoiceConfig;

/// Current supported config schema version.
pub const CURRENT_CONFIG_VERSION: u32 = 2;

/// Migrates a config to the current version.
///
/// - v0 → v1: initial structured config (adds defaults for any missing fields).
/// - v1 → v2: adds `[device]` section (device_id, device_name, pairing_token).
///   Fields are auto-populated by serde(default) when missing from the file.
/// - Unknown future versions: reset to defaults to avoid runtime surprises.
pub fn migrate_config(config: &mut DropVoiceConfig) {
    match config.meta.config_version {
        0 => {
            warn!("Migrating config from v0 to v1");
            config.meta.config_version = 1;
            // Fall through to v1 → v2 migration.
            migrate_v1_to_v2(config);
            config.save_if_dirty();
        }
        1 => {
            warn!("Migrating config from v1 to v2");
            migrate_v1_to_v2(config);
            config.save_if_dirty();
        }
        2 => { /* current version, nothing to do */ }
        version => {
            warn!("Unknown config version {}, resetting to defaults", version);
            *config = DropVoiceConfig::default();
        }
    }
}

/// v1 → v2: the `[device]` section is auto-populated by serde(default).
/// We just bump the version number; serde handles the rest when loading.
fn migrate_v1_to_v2(config: &mut DropVoiceConfig) {
    config.meta.config_version = 2;
}

impl DropVoiceConfig {
    /// Best-effort helper used during migration; failures are only logged.
    fn save_if_dirty(&self) {
        if let Err(e) = self.save() {
            warn!("failed to persist migrated config: {e}");
        }
    }
}
