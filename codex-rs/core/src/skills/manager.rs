use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;

<<<<<<< HEAD
use codex_app_server_protocol::ConfigLayerSource;
=======
use codex_protocol::protocol::Product;
>>>>>>> upstream_main
use codex_protocol::protocol::SkillScope;
use codex_utils_absolute_path::AbsolutePathBuf;
use toml::Value as TomlValue;
use tracing::info;
use tracing::warn;

use crate::config::Config;
use crate::config::types::SkillsConfig;
use crate::config_loader::CloudRequirementsLoader;
use crate::config_loader::ConfigLayerStackOrdering;
use crate::config_loader::LoaderOverrides;
use crate::config_loader::load_config_layers_state;
use crate::plugins::PluginsManager;
use crate::skills::SkillLoadOutcome;
use crate::skills::build_implicit_skill_path_indexes;
use crate::skills::config_rules::SkillConfigRules;
use crate::skills::config_rules::resolve_disabled_skill_paths;
use crate::skills::config_rules::skill_config_rules_from_stack;
use crate::skills::loader::SkillRoot;
use crate::skills::loader::load_skills_from_roots;
use crate::skills::loader::skill_roots;
use crate::skills::system::install_system_skills;
use crate::skills::system::uninstall_system_skills;

pub struct SkillsManager {
    codex_home: PathBuf,
    plugins_manager: Arc<PluginsManager>,
    restriction_product: Option<Product>,
    cache_by_cwd: RwLock<HashMap<PathBuf, SkillLoadOutcome>>,
    cache_by_config: RwLock<HashMap<ConfigSkillsCacheKey, SkillLoadOutcome>>,
}

impl SkillsManager {
    pub fn new(
        codex_home: PathBuf,
        plugins_manager: Arc<PluginsManager>,
        bundled_skills_enabled: bool,
    ) -> Self {
        Self::new_with_restriction_product(
            codex_home,
            plugins_manager,
            bundled_skills_enabled,
            Some(Product::Codex),
        )
    }

    pub fn new_with_restriction_product(
        codex_home: PathBuf,
        plugins_manager: Arc<PluginsManager>,
        bundled_skills_enabled: bool,
        restriction_product: Option<Product>,
    ) -> Self {
        let manager = Self {
            codex_home,
            plugins_manager,
            restriction_product,
            cache_by_cwd: RwLock::new(HashMap::new()),
            cache_by_config: RwLock::new(HashMap::new()),
        };
        if !bundled_skills_enabled {
            // The loader caches bundled skills under `skills/.system`. Clearing that directory is
            // best-effort cleanup; root selection still enforces the config even if removal fails.
            uninstall_system_skills(&manager.codex_home);
        } else if let Err(err) = install_system_skills(&manager.codex_home) {
            tracing::error!("failed to install system skills: {err}");
        }
        manager
    }

    /// Load skills for an already-constructed [`Config`], avoiding any additional config-layer
    /// loading.
    ///
    /// This path uses a cache keyed by the effective skill-relevant config state rather than just
    /// cwd so role-local and session-local skill overrides cannot bleed across sessions that happen
    /// to share a directory.
    pub fn skills_for_config(&self, config: &Config) -> SkillLoadOutcome {
        let roots = self.skill_roots_for_config(config);
        let skill_config_rules = skill_config_rules_from_stack(&config.config_layer_stack);
        let cache_key = config_skills_cache_key(&roots, &skill_config_rules);
        if let Some(outcome) = self.cached_outcome_for_config(&cache_key) {
            return outcome;
        }

        let outcome = self.build_skill_outcome(roots, &skill_config_rules);
        let mut cache = self
            .cache_by_config
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        cache.insert(cache_key, outcome.clone());
        outcome
    }

    pub(crate) fn skill_roots_for_config(&self, config: &Config) -> Vec<SkillRoot> {
        let loaded_plugins = self.plugins_manager.plugins_for_config(config);
        let mut roots = skill_roots(
            &config.config_layer_stack,
            &config.cwd,
            loaded_plugins.effective_skill_roots(),
        );
        if !config.bundled_skills_enabled() {
            roots.retain(|root| root.scope != SkillScope::System);
        }
        roots
    }

    pub async fn skills_for_cwd(
        &self,
        cwd: &Path,
        config: &Config,
        force_reload: bool,
    ) -> SkillLoadOutcome {
        if !force_reload && let Some(outcome) = self.cached_outcome_for_cwd(cwd) {
            return outcome;
        }

        self.skills_for_cwd_with_extra_user_roots(cwd, config, force_reload, &[])
            .await
    }

    pub async fn skills_for_cwd_with_extra_user_roots(
        &self,
        cwd: &Path,
        config: &Config,
        force_reload: bool,
        extra_user_roots: &[PathBuf],
    ) -> SkillLoadOutcome {
        if !force_reload && let Some(outcome) = self.cached_outcome_for_cwd(cwd) {
            return outcome;
        }
        let normalized_extra_user_roots = normalize_extra_user_roots(extra_user_roots);

        let cwd_abs = match AbsolutePathBuf::try_from(cwd) {
            Ok(cwd_abs) => cwd_abs,
            Err(err) => {
                return SkillLoadOutcome {
                    errors: vec![crate::skills::model::SkillError {
                        path: cwd.to_path_buf(),
                        message: err.to_string(),
                    }],
                    ..Default::default()
                };
            }
        };

        let cli_overrides: Vec<(String, TomlValue)> = Vec::new();
        let config_layer_stack = match load_config_layers_state(
            &self.codex_home,
            Some(cwd_abs),
            &cli_overrides,
            LoaderOverrides::default(),
            CloudRequirementsLoader::default(),
        )
        .await
        {
            Ok(config_layer_stack) => config_layer_stack,
            Err(err) => {
                return SkillLoadOutcome {
                    errors: vec![crate::skills::model::SkillError {
                        path: cwd.to_path_buf(),
                        message: err.to_string(),
                    }],
                    ..Default::default()
                };
            }
        };

        let loaded_plugins = self
            .plugins_manager
            .plugins_for_config_with_force_reload(config, force_reload);
        let mut roots = skill_roots(
            &config_layer_stack,
            cwd,
            loaded_plugins.effective_skill_roots(),
        );
        if !bundled_skills_enabled_from_stack(&config_layer_stack) {
            roots.retain(|root| root.scope != SkillScope::System);
        }
        roots.extend(
            normalized_extra_user_roots
                .iter()
                .cloned()
                .map(|path| SkillRoot {
                    path,
                    scope: SkillScope::User,
                }),
        );
<<<<<<< HEAD
        let mut outcome = load_skills_from_roots(roots);
        if !extra_user_roots.is_empty() {
            // When extra user roots are provided, skip system skills before caching the result.
            outcome
                .skills
                .retain(|skill| skill.scope != SkillScope::System);
        }
        outcome.disabled_paths = disabled_paths_from_stack(&config_layer_stack);
        let (by_scripts_dir, by_doc_path) =
            build_implicit_skill_path_indexes(outcome.allowed_skills_for_implicit_invocation());
        outcome.implicit_skills_by_scripts_dir = Arc::new(by_scripts_dir);
        outcome.implicit_skills_by_doc_path = Arc::new(by_doc_path);
        let mut cache = match self.cache_by_cwd.write() {
            Ok(cache) => cache,
            Err(err) => err.into_inner(),
        };
=======
        let skill_config_rules = skill_config_rules_from_stack(&config_layer_stack);
        let outcome = self.build_skill_outcome(roots, &skill_config_rules);
        let mut cache = self
            .cache_by_cwd
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
>>>>>>> upstream_main
        cache.insert(cwd.to_path_buf(), outcome.clone());
        outcome
    }

    fn build_skill_outcome(
        &self,
        roots: Vec<SkillRoot>,
        skill_config_rules: &SkillConfigRules,
    ) -> SkillLoadOutcome {
        let outcome = crate::skills::filter_skill_load_outcome_for_product(
            load_skills_from_roots(roots),
            self.restriction_product,
        );
        let disabled_paths = resolve_disabled_skill_paths(&outcome.skills, skill_config_rules);
        finalize_skill_outcome(outcome, disabled_paths)
    }

    pub fn clear_cache(&self) {
        let cleared_cwd = {
            let mut cache = self
                .cache_by_cwd
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let cleared = cache.len();
            cache.clear();
            cleared
        };
        let cleared_config = {
            let mut cache = self
                .cache_by_config
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let cleared = cache.len();
            cache.clear();
            cleared
        };
        let cleared = cleared_cwd + cleared_config;
        info!("skills cache cleared ({cleared} entries)");
    }

    fn cached_outcome_for_cwd(&self, cwd: &Path) -> Option<SkillLoadOutcome> {
        match self.cache_by_cwd.read() {
            Ok(cache) => cache.get(cwd).cloned(),
            Err(err) => err.into_inner().get(cwd).cloned(),
        }
    }

    fn cached_outcome_for_config(
        &self,
        cache_key: &ConfigSkillsCacheKey,
    ) -> Option<SkillLoadOutcome> {
        match self.cache_by_config.read() {
            Ok(cache) => cache.get(cache_key).cloned(),
            Err(err) => err.into_inner().get(cache_key).cloned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ConfigSkillsCacheKey {
    roots: Vec<(PathBuf, u8)>,
    skill_config_rules: SkillConfigRules,
}

pub(crate) fn bundled_skills_enabled_from_stack(
    config_layer_stack: &crate::config_loader::ConfigLayerStack,
<<<<<<< HEAD
) -> HashSet<PathBuf> {
    let mut disabled = HashSet::new();
    let mut configs = HashMap::new();
    for layer in
        config_layer_stack.get_layers(ConfigLayerStackOrdering::LowestPrecedenceFirst, true)
    {
        if !matches!(
            layer.name,
            ConfigLayerSource::User { .. } | ConfigLayerSource::SessionFlags
        ) {
            continue;
=======
) -> bool {
    let effective_config = config_layer_stack.effective_config();
    let Some(skills_value) = effective_config
        .as_table()
        .and_then(|table| table.get("skills"))
    else {
        return true;
    };

    let skills: SkillsConfig = match skills_value.clone().try_into() {
        Ok(skills) => skills,
        Err(err) => {
            warn!("invalid skills config: {err}");
            return true;
>>>>>>> upstream_main
        }

<<<<<<< HEAD
        let Some(skills_value) = layer.config.get("skills") else {
            continue;
        };
        let skills: SkillsConfig = match skills_value.clone().try_into() {
            Ok(skills) => skills,
            Err(err) => {
                warn!("invalid skills config: {err}");
                continue;
            }
        };

        for entry in skills.config {
            let path = normalize_override_path(entry.path.as_path());
            configs.insert(path, entry.enabled);
        }
    }

    for (path, enabled) in configs {
        if !enabled {
            disabled.insert(path);
        }
    }

    disabled
=======
    skills.bundled.unwrap_or_default().enabled
>>>>>>> upstream_main
}

fn config_skills_cache_key(
    roots: &[SkillRoot],
    skill_config_rules: &SkillConfigRules,
) -> ConfigSkillsCacheKey {
    ConfigSkillsCacheKey {
        roots: roots
            .iter()
            .map(|root| {
                let scope_rank = match root.scope {
                    SkillScope::Repo => 0,
                    SkillScope::User => 1,
                    SkillScope::System => 2,
                    SkillScope::Admin => 3,
                };
                (root.path.clone(), scope_rank)
            })
            .collect(),
        skill_config_rules: skill_config_rules.clone(),
    }
}

fn finalize_skill_outcome(
    mut outcome: SkillLoadOutcome,
    disabled_paths: HashSet<PathBuf>,
) -> SkillLoadOutcome {
    outcome.disabled_paths = disabled_paths;
    let (by_scripts_dir, by_doc_path) =
        build_implicit_skill_path_indexes(outcome.allowed_skills_for_implicit_invocation());
    outcome.implicit_skills_by_scripts_dir = Arc::new(by_scripts_dir);
    outcome.implicit_skills_by_doc_path = Arc::new(by_doc_path);
    outcome
}

fn normalize_extra_user_roots(extra_user_roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut normalized: Vec<PathBuf> = extra_user_roots
        .iter()
        .map(|path| dunce::canonicalize(path).unwrap_or_else(|_| path.clone()))
        .collect();
    normalized.sort_unstable();
    normalized.dedup();
    normalized
}

#[cfg(test)]
<<<<<<< HEAD
mod tests {
    use super::*;
    use crate::config::ConfigBuilder;
    use crate::config::ConfigOverrides;
    use crate::config_loader::ConfigLayerEntry;
    use crate::config_loader::ConfigLayerStack;
    use crate::config_loader::ConfigRequirementsToml;
    use pretty_assertions::assert_eq;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn write_user_skill(codex_home: &TempDir, dir: &str, name: &str, description: &str) {
        let skill_dir = codex_home.path().join("skills").join(dir);
        fs::create_dir_all(&skill_dir).unwrap();
        let content = format!("---\nname: {name}\ndescription: {description}\n---\n\n# Body\n");
        fs::write(skill_dir.join("SKILL.md"), content).unwrap();
    }

    #[tokio::test]
    async fn skills_for_config_seeds_cache_by_cwd() {
        let codex_home = tempfile::tempdir().expect("tempdir");
        let cwd = tempfile::tempdir().expect("tempdir");

        let cfg = ConfigBuilder::default()
            .codex_home(codex_home.path().to_path_buf())
            .harness_overrides(ConfigOverrides {
                cwd: Some(cwd.path().to_path_buf()),
                ..Default::default()
            })
            .build()
            .await
            .expect("defaults for test should always succeed");

        let skills_manager = SkillsManager::new(codex_home.path().to_path_buf());

        write_user_skill(&codex_home, "a", "skill-a", "from a");
        let outcome1 = skills_manager.skills_for_config(&cfg);
        assert!(
            outcome1.skills.iter().any(|s| s.name == "skill-a"),
            "expected skill-a to be discovered"
        );

        // Write a new skill after the first call; the second call should hit the cache and not
        // reflect the new file.
        write_user_skill(&codex_home, "b", "skill-b", "from b");
        let outcome2 = skills_manager.skills_for_config(&cfg);
        assert_eq!(outcome2.errors, outcome1.errors);
        assert_eq!(outcome2.skills, outcome1.skills);
    }

    #[tokio::test]
    async fn skills_for_cwd_reuses_cached_entry_even_when_entry_has_extra_roots() {
        let codex_home = tempfile::tempdir().expect("tempdir");
        let cwd = tempfile::tempdir().expect("tempdir");
        let extra_root = tempfile::tempdir().expect("tempdir");

        let config = ConfigBuilder::default()
            .codex_home(codex_home.path().to_path_buf())
            .harness_overrides(ConfigOverrides {
                cwd: Some(cwd.path().to_path_buf()),
                ..Default::default()
            })
            .build()
            .await
            .expect("defaults for test should always succeed");

        let skills_manager = SkillsManager::new(codex_home.path().to_path_buf());
        let _ = skills_manager.skills_for_config(&config);

        write_user_skill(&extra_root, "x", "extra-skill", "from extra root");
        let extra_root_path = extra_root.path().to_path_buf();
        let outcome_with_extra = skills_manager
            .skills_for_cwd_with_extra_user_roots(
                cwd.path(),
                true,
                std::slice::from_ref(&extra_root_path),
            )
            .await;
        assert!(
            outcome_with_extra
                .skills
                .iter()
                .any(|skill| skill.name == "extra-skill")
        );

        // The cwd-only API returns the current cached entry for this cwd, even when that entry
        // was produced with extra roots.
        let outcome_without_extra = skills_manager.skills_for_cwd(cwd.path(), false).await;
        assert_eq!(outcome_without_extra.skills, outcome_with_extra.skills);
        assert_eq!(outcome_without_extra.errors, outcome_with_extra.errors);
    }

    #[tokio::test]
    async fn skills_for_cwd_with_extra_roots_only_refreshes_on_force_reload() {
        let codex_home = tempfile::tempdir().expect("tempdir");
        let cwd = tempfile::tempdir().expect("tempdir");
        let extra_root_a = tempfile::tempdir().expect("tempdir");
        let extra_root_b = tempfile::tempdir().expect("tempdir");

        let config = ConfigBuilder::default()
            .codex_home(codex_home.path().to_path_buf())
            .harness_overrides(ConfigOverrides {
                cwd: Some(cwd.path().to_path_buf()),
                ..Default::default()
            })
            .build()
            .await
            .expect("defaults for test should always succeed");

        let skills_manager = SkillsManager::new(codex_home.path().to_path_buf());
        let _ = skills_manager.skills_for_config(&config);

        write_user_skill(&extra_root_a, "x", "extra-skill-a", "from extra root a");
        write_user_skill(&extra_root_b, "x", "extra-skill-b", "from extra root b");

        let extra_root_a_path = extra_root_a.path().to_path_buf();
        let outcome_a = skills_manager
            .skills_for_cwd_with_extra_user_roots(
                cwd.path(),
                true,
                std::slice::from_ref(&extra_root_a_path),
            )
            .await;
        assert!(
            outcome_a
                .skills
                .iter()
                .any(|skill| skill.name == "extra-skill-a")
        );
        assert!(
            outcome_a
                .skills
                .iter()
                .all(|skill| skill.name != "extra-skill-b")
        );

        let extra_root_b_path = extra_root_b.path().to_path_buf();
        let outcome_b = skills_manager
            .skills_for_cwd_with_extra_user_roots(
                cwd.path(),
                false,
                std::slice::from_ref(&extra_root_b_path),
            )
            .await;
        assert!(
            outcome_b
                .skills
                .iter()
                .any(|skill| skill.name == "extra-skill-a")
        );
        assert!(
            outcome_b
                .skills
                .iter()
                .all(|skill| skill.name != "extra-skill-b")
        );

        let outcome_reloaded = skills_manager
            .skills_for_cwd_with_extra_user_roots(
                cwd.path(),
                true,
                std::slice::from_ref(&extra_root_b_path),
            )
            .await;
        assert!(
            outcome_reloaded
                .skills
                .iter()
                .any(|skill| skill.name == "extra-skill-b")
        );
        assert!(
            outcome_reloaded
                .skills
                .iter()
                .all(|skill| skill.name != "extra-skill-a")
        );
    }

    #[test]
    fn normalize_extra_user_roots_is_stable_for_equivalent_inputs() {
        let a = PathBuf::from("/tmp/a");
        let b = PathBuf::from("/tmp/b");

        let first = normalize_extra_user_roots(&[a.clone(), b.clone(), a.clone()]);
        let second = normalize_extra_user_roots(&[b, a]);

        assert_eq!(first, second);
    }

    #[cfg_attr(windows, ignore)]
    #[test]
    fn disabled_paths_from_stack_allows_session_flags_to_override_user_layer() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let skill_path = tempdir.path().join("skills").join("demo").join("SKILL.md");
        let user_file = AbsolutePathBuf::try_from(tempdir.path().join("config.toml"))
            .expect("user config path should be absolute");
        let user_layer = ConfigLayerEntry::new(
            ConfigLayerSource::User { file: user_file },
            toml::from_str(&format!(
                r#"[[skills.config]]
path = "{}"
enabled = false
"#,
                skill_path.display()
            ))
            .expect("user layer toml"),
        );
        let session_layer = ConfigLayerEntry::new(
            ConfigLayerSource::SessionFlags,
            toml::from_str(&format!(
                r#"[[skills.config]]
path = "{}"
enabled = true
"#,
                skill_path.display()
            ))
            .expect("session layer toml"),
        );
        let stack = ConfigLayerStack::new(
            vec![user_layer, session_layer],
            Default::default(),
            ConfigRequirementsToml::default(),
        )
        .expect("valid config layer stack");

        assert_eq!(disabled_paths_from_stack(&stack), HashSet::new());
    }

    #[cfg_attr(windows, ignore)]
    #[test]
    fn disabled_paths_from_stack_allows_session_flags_to_disable_user_enabled_skill() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let skill_path = tempdir.path().join("skills").join("demo").join("SKILL.md");
        let user_file = AbsolutePathBuf::try_from(tempdir.path().join("config.toml"))
            .expect("user config path should be absolute");
        let user_layer = ConfigLayerEntry::new(
            ConfigLayerSource::User { file: user_file },
            toml::from_str(&format!(
                r#"[[skills.config]]
path = "{}"
enabled = true
"#,
                skill_path.display()
            ))
            .expect("user layer toml"),
        );
        let session_layer = ConfigLayerEntry::new(
            ConfigLayerSource::SessionFlags,
            toml::from_str(&format!(
                r#"[[skills.config]]
path = "{}"
enabled = false
"#,
                skill_path.display()
            ))
            .expect("session layer toml"),
        );
        let stack = ConfigLayerStack::new(
            vec![user_layer, session_layer],
            Default::default(),
            ConfigRequirementsToml::default(),
        )
        .expect("valid config layer stack");

        assert_eq!(
            disabled_paths_from_stack(&stack),
            HashSet::from([skill_path])
        );
    }
}
=======
#[path = "manager_tests.rs"]
mod tests;
>>>>>>> upstream_main
