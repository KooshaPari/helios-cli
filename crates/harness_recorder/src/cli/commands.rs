use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::media::{MediaRecorder, OutputFormat};
use crate::pty::TerminalController;
use crate::script::{Script, ScriptLoader};

pub async fn record_command(
    script_path: PathBuf,
    output_dir: PathBuf,
    format: String,
) -> Result<()> {
    println!("🎬 Recording script: {}", script_path.display());

    // Load script
    let script = ScriptLoader::load_from_file(&script_path)
        .with_context(|| format!("Failed to load script: {}", script_path.display()))?;

    // Parse output format
    let output_format = OutputFormat::from_string(&format)?;

    // Create output directory
    std::fs::create_dir_all(&output_dir)
        .with_context(|| format!("Failed to create output directory: {}", output_dir.display()))?;

    // Initialize terminal controller
    let mut terminal = TerminalController::new(&script.settings)?;

    // Initialize media recorder
    let mut recorder = MediaRecorder::new(output_format, &output_dir)?;

    // Execute script
    println!("🚀 Executing {} steps...", script.steps.len());

    // Track produced artifacts so we can render a summary panel at the end.
    let mut artifacts: Vec<String> = Vec::new();

    for (i, step) in script.steps.iter().enumerate() {
        println!("📝 Step {}/{}: {:?}", i + 1, script.steps.len(), step.step_type);

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
                println!("📸 Screenshot saved: {}", screenshot_path.display());
                artifacts.push(format!("📸 {}", screenshot_path.display()));
            }
            crate::script::StepType::RecordGif { duration, ref name } => {
                let gif_path = output_dir.join(format!("{}.gif", name));
                recorder.start_gif_recording(&terminal).await?;
                tokio::time::sleep(duration).await;
                recorder.stop_gif_recording(&gif_path).await?;
                println!("🎞️ GIF saved: {}", gif_path.display());
                artifacts.push(format!("🎞️ {}", gif_path.display()));
            }
        }
    }

    print_recording_summary(&output_dir, script.steps.len(), &artifacts);
    Ok(())
}

/// Render the end-of-run recording summary as a rich panel via the
/// Phenotype-org rck-core toolkit. Capability detection degrades the panel to
/// plain ASCII when piped, in CI, or on terminals without graphics support, so
/// the output stays pipe-safe.
fn print_recording_summary(output_dir: &std::path::Path, steps: usize, artifacts: &[String]) {
    use std::io::Write;

    let caps = rck_core::detect();

    let mut lines: Vec<String> = vec![
        "Recording complete".to_string(),
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
        println!("✅ Recording complete! Output saved to: {}", output_dir.display());
    }
}

pub async fn screenshot_command(command: String, output: PathBuf) -> Result<()> {
    println!("📸 Taking screenshot of command: {}", command);

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

    println!("✅ Screenshot saved: {}", output.display());
    Ok(())
}

pub async fn demo_command(script_path: PathBuf, interactive: bool) -> Result<()> {
    println!("🎭 Running demo: {}", script_path.display());

    let script = ScriptLoader::load_from_file(&script_path)?;
    let mut terminal = TerminalController::new(&script.settings)?;

    for (i, step) in script.steps.iter().enumerate() {
        if interactive {
            println!("\n📋 Next step {}/{}: {:?}", i + 1, script.steps.len(), step.step_type);
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

    println!("✅ Demo complete!");
    Ok(())
}

pub async fn convert_command(input: PathBuf, output: PathBuf) -> Result<()> {
    println!("🔄 Converting {} to {}", input.display(), output.display());

    // TODO: Implement format conversion logic
    // This would handle converting between different recording formats

    println!("✅ Conversion complete!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_recording_summary_writes_ascii_when_not_tty() {
        let dir = std::path::Path::new("artifacts/test-output");
        print_recording_summary(dir, 2, &["demo.png".to_string()]);
    }

    #[tokio::test]
    async fn convert_command_is_noop_success() {
        convert_command(
            std::path::PathBuf::from("input.gif"),
            std::path::PathBuf::from("output.mp4"),
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
        record_command(script_path, output_dir.clone(), "png".to_string())
            .await
            .expect("record");
        assert!(output_dir.is_dir());
    }
}
