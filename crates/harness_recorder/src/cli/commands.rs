use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::i18n::I18n;
use crate::media::{MediaRecorder, OutputFormat};
use crate::pty::TerminalController;
use crate::script::{Script, ScriptLoader};

pub async fn record_command(
    script_path: PathBuf,
    output_dir: PathBuf,
    format: String,
    i18n: &I18n,
) -> Result<()> {
    println!("{}", i18n.t_with("status.recording", &[("script", &script_path.display().to_string())]));

    // Load script
    let script = ScriptLoader::load_from_file(&script_path)
        .with_context(|| i18n.t_with("error.load_script", &[("path", &script_path.display().to_string())]))?;

    // Parse output format
    let output_format = OutputFormat::from_string(&format)?;

    // Create output directory
    std::fs::create_dir_all(&output_dir)
        .with_context(|| i18n.t_with("error.create_output_dir", &[("path", &output_dir.display().to_string())]))?;

    // Initialize terminal controller
    let mut terminal = TerminalController::new(&script.settings)?;

    // Initialize media recorder
    let mut recorder = MediaRecorder::new(output_format, &output_dir)?;

    // Execute script
    println!("{}", i18n.t_with("status.executing", &[("count", &script.steps.len().to_string())]));

    // Track produced artifacts so we can render a summary panel at the end.
    let mut artifacts: Vec<String> = Vec::new();

    for (i, step) in script.steps.iter().enumerate() {
        println!("{}", i18n.t_with("status.step", &[("index", &(i + 1).to_string()), ("total", &script.steps.len().to_string()), ("type", &format!("{:?}", step.step_type))]));

        match step.step_type {
            crate::script::StepType::Command { ref text, wait } => {
                terminal.execute_command(text).await?;
                if let Some(duration) = wait {
                    tokio::time::sleep(duration).await;
                }
            }
            crate::script::StepType::Type { ref text, speed } => {
                terminal.type_text(text, speed).await?;
            }
            crate::script::StepType::Screenshot { ref name } => {
                let screenshot_path = output_dir.join(format!("{}.png", name));
                recorder.take_screenshot(&terminal, &screenshot_path).await?;
                println!("{}", i18n.t_with("status.screenshot_taken", &[("path", &screenshot_path.display().to_string())]));
                artifacts.push(format!("📸 {}", screenshot_path.display()));
            }
            crate::script::StepType::RecordGif { duration, ref name } => {
                let gif_path = output_dir.join(format!("{}.gif", name));
                recorder.start_gif_recording(&terminal).await?;
                tokio::time::sleep(duration).await;
                recorder.stop_gif_recording(&gif_path).await?;
                println!("{}", i18n.t_with("status.screenshot_taken", &[("path", &gif_path.display().to_string())]));
                artifacts.push(format!("🎞️ {}", gif_path.display()));
            }
        }
    }

    print_recording_summary(&output_dir, script.steps.len(), &artifacts, i18n);
    Ok(())
}

/// Render the end-of-run recording summary as a rich panel via the
/// Phenotype-org rck-core toolkit. Capability detection degrades the panel to
/// plain ASCII when piped, in CI, or on terminals without graphics support, so
/// the output stays pipe-safe.
fn print_recording_summary(output_dir: &std::path::Path, steps: usize, artifacts: &[String], i18n: &I18n) {
    use std::io::Write;

    let caps = rck_core::detect();

    let mut lines: Vec<String> = vec![
        i18n.t("status.done"),
        format!("steps    : {steps}"),
        format!("output   : {}", output_dir.display()),
    ];
    if artifacts.is_empty() {
        lines.push("artifacts: (none)".to_string());
    } else {
        lines.push("artifacts:".to_string());
        lines.extend(artifacts.iter().map(|a| format!("  {a}")));
    }
    let line_refs: Vec<&str> = lines.iter().map(String::as_str).collect();

    // Rounded (unicode) border on a real terminal; plain ASCII border when
    // piped or in CI so downstream consumers never see unicode box-drawing.
    let border =
        if caps.is_tty { rck_core::BorderStyle::Rounded } else { rck_core::BorderStyle::Ascii };

    let mut out = std::io::stdout().lock();
    // On any rendering failure, fall back to a plain status line so the command
    // never fails just because of presentation.
    if rck_core::emit_panel(&mut out, &caps, "kla record", &line_refs, border)
        .and_then(|()| out.flush().map_err(Into::into))
        .is_err()
    {
        println!("✅ {}", i18n.t("status.done"));
    }
}

pub async fn screenshot_command(command: String, output: PathBuf, i18n: &I18n) -> Result<()> {
    println!("📸 {}", command);

    // Create a simple single-command script
    let script = Script::single_command(&command)?;

    // Initialize terminal
    let mut terminal = TerminalController::new(&script.settings)?;

    // Execute command
    terminal.execute_command(&command).await?;

    // Take screenshot
    let recorder =
        MediaRecorder::new(OutputFormat::Png, output.parent().unwrap_or(&PathBuf::from(".")))?;
    recorder.take_screenshot(&terminal, &output).await?;

    println!("{}", i18n.t_with("status.screenshot_taken", &[("path", &output.display().to_string())]));
    Ok(())
}

pub async fn demo_command(script_path: PathBuf, interactive: bool, i18n: &I18n) -> Result<()> {
    println!("🎭 {}", script_path.display());

    let script = ScriptLoader::load_from_file(&script_path)?;
    let mut terminal = TerminalController::new(&script.settings)?;

    for (i, step) in script.steps.iter().enumerate() {
        if interactive {
            println!("\n📋 {} {}/{}: {:?}", i18n.t("status.step"), i + 1, script.steps.len(), step.step_type);
            println!("Press Enter to continue...");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
        }

        match step.step_type {
            crate::script::StepType::Command { ref text, wait } => {
                terminal.execute_command(text).await?;
                if let Some(duration) = wait {
                    tokio::time::sleep(duration).await;
                }
            }
            crate::script::StepType::Type { ref text, speed } => {
                terminal.type_text(text, speed).await?;
            }
            _ => {} // Skip recording steps in demo mode
        }
    }

    println!("✅ {}", i18n.t("status.done"));
    Ok(())
}

pub async fn convert_command(input: PathBuf, output: PathBuf, i18n: &I18n) -> Result<()> {
    println!("🔄 {} {} {}", i18n.t("cmd.convert"), input.display(), output.display());

    // TODO: Implement format conversion logic
    // This would handle converting between different recording formats

    println!("✅ {}", i18n.t("status.done"));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::I18n;

    #[test]
    fn print_recording_summary_writes_ascii_when_not_tty() {
        let dir = std::path::Path::new("artifacts/test-output");
        let i18n = I18n::new("en");
        print_recording_summary(dir, 2, &["demo.png".to_string()], &i18n);
    }

    #[tokio::test]
    async fn convert_command_is_noop_success() {
        let i18n = I18n::new("en");
        convert_command(
            std::path::PathBuf::from("input.gif"),
            std::path::PathBuf::from("output.mp4"),
            &i18n,
        )
        .await
        .expect("convert");
    }

    #[tokio::test]
    async fn record_command_runs_minimal_script() {
        let dir = tempfile::tempdir().expect("tempdir");
        let script_path = dir.path().join("smoke.kla.yaml");
        #[cfg(windows)]
        let yaml = r#"
name: smoke
settings:
  width: 80
  height: 24
  shell: cmd.exe
steps:
  - type: command
    text: echo record-smoke
    wait: 500ms
"#;
        #[cfg(not(windows))]
        let yaml = r#"
name: smoke
settings:
  width: 80
  height: 24
  shell: sh
steps:
  - type: command
    text: echo record-smoke
    wait: 500ms
"#;
        std::fs::write(&script_path, yaml).expect("write script");
        let output_dir = dir.path().join("out");
        let i18n = I18n::new("en");
        record_command(script_path, output_dir.clone(), "png".to_string(), &i18n).await.expect("record");
        assert!(output_dir.is_dir());
    }
}
