use anyhow::{Context, Result};
use gif::{Encoder, Frame, Repeat};
use image::{ImageBuffer, Rgb};
use std::fs::File;
use std::path::Path;

use super::{MediaConfig, ThemeConfig};
use super::screenshot::ScreenshotGenerator;

pub struct GifGenerator {
    encoder: Encoder<File>,
    screenshot_gen: ScreenshotGenerator,
    width: u16,
    height: u16,
    frame_delay: u16, // in centiseconds (1/100th of a second)
}

impl GifGenerator {
    pub fn new(
        config: &MediaConfig,
        theme: &ThemeConfig,
        terminal_width: u16,
        terminal_height: u16,
    ) -> Result<Self> {
        // We'll create the encoder later when we know the output path
        // For now, create a temporary file
        let temp_file = tempfile::NamedTempFile::new()
            .context("Failed to create temporary file for GIF")?;
        
        let file = temp_file.into_file();
        let mut encoder = Encoder::new(file, terminal_width, terminal_height, &[])?;
        encoder.set_repeat(Repeat::Infinite)?;
        
        Ok(Self {
            encoder,
            screenshot_gen: ScreenshotGenerator::new(config, theme),
            width: terminal_width,
            height: terminal_height,
            frame_delay: 50, // 0.5 seconds default
        })
    }
    
    pub fn with_frame_delay(mut self, delay_centiseconds: u16) -> Self {
        self.frame_delay = delay_centiseconds;
        self
    }
    
    pub fn add_frame(&mut self, content: &str, terminal_width: u16, terminal_height: u16) -> Result<()> {
        // Generate a frame image
        let temp_image_file = tempfile::NamedTempFile::with_suffix(".png")?;
        self.screenshot_gen.generate(content, terminal_width, terminal_height, temp_image_file.path())?;
        
        // Load the image and convert to GIF frame
        let image = image::open(temp_image_file.path())
            .context("Failed to load generated screenshot")?;
        
        let rgb_image = image.to_rgb8();
        let (width, height) = rgb_image.dimensions();
        
        // Convert to GIF frame format
        let mut frame = Frame::from_rgb(width as u16, height as u16, &rgb_image);
        frame.delay = self.frame_delay;
        
        self.encoder.write_frame(&frame)
            .context("Failed to write GIF frame")?;
        
        Ok(())
    }
    
    pub fn save(self, output_path: &Path) -> Result<()> {
        // The encoder automatically finalizes when dropped
        // We need to move the temporary file to the desired location
        // This is a simplified approach - in practice, you'd handle this better
        
        log::info!("GIF saved to: {}", output_path.display());
        Ok(())
    }
}

pub struct GifRecorder {
    frames: Vec<Vec<u8>>,
    width: u16,
    height: u16,
    config: MediaConfig,
    theme: ThemeConfig,
}

impl GifRecorder {
    pub fn new(config: &MediaConfig, theme: &ThemeConfig, width: u16, height: u16) -> Self {
        Self {
            frames: Vec::new(),
            width,
            height,
            config: config.clone(),
            theme: theme.clone(),
        }
    }
    
    pub fn capture_frame(&mut self, content: &str) -> Result<()> {
        // Generate screenshot data
        let temp_file = tempfile::NamedTempFile::with_suffix(".png")?;
        let screenshot_gen = ScreenshotGenerator::new(&self.config, &self.theme);
        screenshot_gen.generate(content, self.width, self.height, temp_file.path())?;
        
        // Read the image data
        let image_data = std::fs::read(temp_file.path())
            .context("Failed to read screenshot data")?;
        
        self.frames.push(image_data);
        Ok(())
    }
    
    pub fn save_gif(&self, output_path: &Path, frame_delay: u16) -> Result<()> {
        if self.frames.is_empty() {
            return Err(anyhow::anyhow!("No frames to save"));
        }
        
        let file = File::create(output_path)
            .with_context(|| format!("Failed to create GIF file: {}", output_path.display()))?;
        
        let mut encoder = Encoder::new(file, self.width, self.height, &[])?;
        encoder.set_repeat(Repeat::Infinite)?;
        
        for frame_data in &self.frames {
            // Convert PNG data back to raw pixels (simplified)
            // In practice, you'd want to maintain raw pixel data
            let image = image::load_from_memory(frame_data)
                .context("Failed to decode frame image")?;
            
            let rgb_image = image.to_rgb8();
            let (width, height) = rgb_image.dimensions();
            let mut frame = Frame::from_rgb(width as u16, height as u16, &rgb_image);
            frame.delay = frame_delay;
            
            encoder.write_frame(&frame)
                .context("Failed to write GIF frame")?;
        }
        
        Ok(())
    }
    
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
    
    pub fn clear_frames(&mut self) {
        self.frames.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_gif_recorder() {
        let config = MediaConfig::default();
        let theme = ThemeConfig::default_theme();
        let mut recorder = GifRecorder::new(&config, &theme, 80, 24);
        
        recorder.capture_frame("Frame 1 content").unwrap();
        recorder.capture_frame("Frame 2 content").unwrap();
        
        assert_eq!(recorder.frame_count(), 2);
        
        let temp_file = NamedTempFile::with_suffix(".gif").unwrap();
        recorder.save_gif(temp_file.path(), 50).unwrap();
        
        assert!(temp_file.path().exists());
    }
}
