//! Shared helpers for filtering and matching built-in slash commands.
//!
//! The same sandbox- and feature-gating rules are used by both the composer
//! and the command popup. Centralizing them here keeps those call sites small
//! and ensures they stay in sync.
use std::str::FromStr;

use codex_utils_fuzzy_match::fuzzy_match;

use crate::slash_command::SlashCommand;
use crate::slash_command::built_in_slash_commands;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct BuiltinCommandFlags {
    pub(crate) collaboration_modes_enabled: bool,
    pub(crate) connectors_enabled: bool,
    pub(crate) plugins_command_enabled: bool,
    pub(crate) fast_command_enabled: bool,
    pub(crate) personality_command_enabled: bool,
    pub(crate) realtime_conversation_enabled: bool,
    pub(crate) audio_device_selection_enabled: bool,
    pub(crate) allow_elevate_sandbox: bool,
}

/// Return the built-ins that should be visible/usable for the current input.
<<<<<<< HEAD
pub(crate) fn builtins_for_input(
    collaboration_modes_enabled: bool,
    connectors_enabled: bool,
    personality_command_enabled: bool,
    realtime_conversation_enabled: bool,
    audio_device_selection_enabled: bool,
    allow_elevate_sandbox: bool,
) -> Vec<(&'static str, SlashCommand)> {
=======
pub(crate) fn builtins_for_input(flags: BuiltinCommandFlags) -> Vec<(&'static str, SlashCommand)> {
>>>>>>> upstream_main
    built_in_slash_commands()
        .into_iter()
        .filter(|(_, cmd)| flags.allow_elevate_sandbox || *cmd != SlashCommand::ElevateSandbox)
        .filter(|(_, cmd)| {
            flags.collaboration_modes_enabled
                || !matches!(*cmd, SlashCommand::Collab | SlashCommand::Plan)
        })
<<<<<<< HEAD
        .filter(|(_, cmd)| connectors_enabled || *cmd != SlashCommand::Apps)
        .filter(|(_, cmd)| personality_command_enabled || *cmd != SlashCommand::Personality)
        .filter(|(_, cmd)| realtime_conversation_enabled || *cmd != SlashCommand::Realtime)
        .filter(|(_, cmd)| audio_device_selection_enabled || *cmd != SlashCommand::Settings)
=======
        .filter(|(_, cmd)| flags.connectors_enabled || *cmd != SlashCommand::Apps)
        .filter(|(_, cmd)| flags.plugins_command_enabled || *cmd != SlashCommand::Plugins)
        .filter(|(_, cmd)| flags.fast_command_enabled || *cmd != SlashCommand::Fast)
        .filter(|(_, cmd)| flags.personality_command_enabled || *cmd != SlashCommand::Personality)
        .filter(|(_, cmd)| flags.realtime_conversation_enabled || *cmd != SlashCommand::Realtime)
        .filter(|(_, cmd)| flags.audio_device_selection_enabled || *cmd != SlashCommand::Settings)
>>>>>>> upstream_main
        .collect()
}

/// Find a single built-in command by exact name, after applying the gating rules.
<<<<<<< HEAD
pub(crate) fn find_builtin_command(
    name: &str,
    collaboration_modes_enabled: bool,
    connectors_enabled: bool,
    personality_command_enabled: bool,
    realtime_conversation_enabled: bool,
    audio_device_selection_enabled: bool,
    allow_elevate_sandbox: bool,
) -> Option<SlashCommand> {
    builtins_for_input(
        collaboration_modes_enabled,
        connectors_enabled,
        personality_command_enabled,
        realtime_conversation_enabled,
        audio_device_selection_enabled,
        allow_elevate_sandbox,
    )
    .into_iter()
    .find(|(command_name, _)| *command_name == name)
    .map(|(_, cmd)| cmd)
}

/// Whether any visible built-in fuzzily matches the provided prefix.
pub(crate) fn has_builtin_prefix(
    name: &str,
    collaboration_modes_enabled: bool,
    connectors_enabled: bool,
    personality_command_enabled: bool,
    realtime_conversation_enabled: bool,
    audio_device_selection_enabled: bool,
    allow_elevate_sandbox: bool,
) -> bool {
    builtins_for_input(
        collaboration_modes_enabled,
        connectors_enabled,
        personality_command_enabled,
        realtime_conversation_enabled,
        audio_device_selection_enabled,
        allow_elevate_sandbox,
    )
    .into_iter()
    .any(|(command_name, _)| fuzzy_match(command_name, name).is_some())
=======
pub(crate) fn find_builtin_command(name: &str, flags: BuiltinCommandFlags) -> Option<SlashCommand> {
    let cmd = SlashCommand::from_str(name).ok()?;
    builtins_for_input(flags)
        .into_iter()
        .any(|(_, visible_cmd)| visible_cmd == cmd)
        .then_some(cmd)
}

/// Whether any visible built-in fuzzily matches the provided prefix.
pub(crate) fn has_builtin_prefix(name: &str, flags: BuiltinCommandFlags) -> bool {
    builtins_for_input(flags)
        .into_iter()
        .any(|(command_name, _)| fuzzy_match(command_name, name).is_some())
>>>>>>> upstream_main
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn all_enabled_flags() -> BuiltinCommandFlags {
        BuiltinCommandFlags {
            collaboration_modes_enabled: true,
            connectors_enabled: true,
            plugins_command_enabled: true,
            fast_command_enabled: true,
            personality_command_enabled: true,
            realtime_conversation_enabled: true,
            audio_device_selection_enabled: true,
            allow_elevate_sandbox: true,
        }
    }

    #[test]
    fn debug_command_still_resolves_for_dispatch() {
<<<<<<< HEAD
        let cmd = find_builtin_command("debug-config", true, true, true, false, false, false);
=======
        let cmd = find_builtin_command("debug-config", all_enabled_flags());
>>>>>>> upstream_main
        assert_eq!(cmd, Some(SlashCommand::DebugConfig));
    }

    #[test]
    fn clear_command_resolves_for_dispatch() {
        assert_eq!(
<<<<<<< HEAD
            find_builtin_command("clear", true, true, true, false, false, false),
=======
            find_builtin_command("clear", all_enabled_flags()),
>>>>>>> upstream_main
            Some(SlashCommand::Clear)
        );
    }

    #[test]
    fn stop_command_resolves_for_dispatch() {
        assert_eq!(
<<<<<<< HEAD
            find_builtin_command("realtime", true, true, true, false, true, false),
            None
        );
    }

    #[test]
    fn settings_command_is_hidden_when_realtime_is_disabled() {
        assert_eq!(
            find_builtin_command("settings", true, true, true, false, false, false),
            None
        );
    }

    #[test]
    fn settings_command_is_hidden_when_audio_device_selection_is_disabled() {
        assert_eq!(
            find_builtin_command("settings", true, true, true, true, false, false),
            None
=======
            find_builtin_command("stop", all_enabled_flags()),
            Some(SlashCommand::Stop)
>>>>>>> upstream_main
        );
    }

    #[test]
    fn clean_command_alias_resolves_for_dispatch() {
        assert_eq!(
            find_builtin_command("clean", all_enabled_flags()),
            Some(SlashCommand::Stop)
        );
    }

    #[test]
    fn fast_command_is_hidden_when_disabled() {
        let mut flags = all_enabled_flags();
        flags.fast_command_enabled = false;
        assert_eq!(find_builtin_command("fast", flags), None);
    }

    #[test]
    fn realtime_command_is_hidden_when_realtime_is_disabled() {
        let mut flags = all_enabled_flags();
        flags.realtime_conversation_enabled = false;
        assert_eq!(find_builtin_command("realtime", flags), None);
    }

    #[test]
    fn settings_command_is_hidden_when_realtime_is_disabled() {
        let mut flags = all_enabled_flags();
        flags.realtime_conversation_enabled = false;
        flags.audio_device_selection_enabled = false;
        assert_eq!(find_builtin_command("settings", flags), None);
    }

    #[test]
    fn settings_command_is_hidden_when_audio_device_selection_is_disabled() {
        let mut flags = all_enabled_flags();
        flags.audio_device_selection_enabled = false;
        assert_eq!(find_builtin_command("settings", flags), None);
    }
}
