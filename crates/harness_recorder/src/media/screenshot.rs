use anyhow::{Context, Result};
use image::{ImageBuffer, Rgb, RgbImage};
use std::path::Path;

use super::{MediaConfig, ThemeConfig, MediaGenerator};

pub struct ScreenshotGenerator {
    config: MediaConfig,
    theme: ThemeConfig,
}

impl ScreenshotGenerator {
    pub fn new(config: &MediaConfig, theme: &ThemeConfig) -> Self {
        Self {
            config: config.clone(),
            theme: theme.clone(),
        }
    }
    
    pub fn generate(
        &self,
        content: &str,
        terminal_width: u16,
        terminal_height: u16,
        output_path: &Path,
    ) -> Result<()> {
        // Calculate image dimensions
        let char_width = self.config.font_size as u32 * 6 / 10; // Approximate monospace width
        let char_height = (self.config.font_size as f32 * self.config.line_height) as u32;
        
        let image_width = (terminal_width as u32 * char_width) + (self.config.padding as u32 * 2);
        let image_height = (terminal_height as u32 * char_height) + (self.config.padding as u32 * 2);
        
        // Create image
        let mut image: RgbImage = ImageBuffer::new(image_width, image_height);
        
        // Fill background
        let bg_color = Rgb([
            self.theme.background.0,
            self.theme.background.1,
            self.theme.background.2,
        ]);
        
        for pixel in image.pixels_mut() {
            *pixel = bg_color;
        }
        
        // Render text (simplified - in a real implementation, we'd need proper font rendering)
        self.render_terminal_content(&mut image, content, terminal_width, terminal_height)?;
        
        // Save image
        image.save(output_path)
            .with_context(|| format!("Failed to save screenshot to: {}", output_path.display()))?;
        
        Ok(())
    }
    
    fn render_terminal_content(
        &self,
        image: &mut RgbImage,
        content: &str,
        terminal_width: u16,
        terminal_height: u16,
    ) -> Result<()> {
        // This is a simplified text rendering
        // In a production implementation, you'd use a proper font rendering library
        // like rusttype or fontdue to render actual text
        
        let lines: Vec<&str> = content.lines().collect();
        let char_width = self.config.font_size as u32 * 6 / 10;
        let char_height = (self.config.font_size as f32 * self.config.line_height) as u32;
        
        let text_color = Rgb([
            self.theme.foreground.0,
            self.theme.foreground.1,
            self.theme.foreground.2,
        ]);
        
        for (line_idx, line) in lines.iter().enumerate().take(terminal_height as usize) {
            let y_offset = self.config.padding as u32 + (line_idx as u32 * char_height);
            
            for (char_idx, _ch) in line.chars().enumerate().take(terminal_width as usize) {
                let x_offset = self.config.padding as u32 + (char_idx as u32 * char_width);
                
                // Simple character rendering (just a colored rectangle for now)
                // In real implementation, render actual glyphs
                self.draw_char_placeholder(image, x_offset, y_offset, char_width, char_height, text_color);
            }
        }
        
        Ok(())
    }
    
    fn draw_char_placeholder(
        &self,
        image: &mut RgbImage,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        color: Rgb<u8>,
    ) {
        for dy in 0..height.min(4) { // Just draw a small rectangle as placeholder
            for dx in 0..width.min(2) {
                if x + dx < image.width() && y + dy < image.height() {
                    image.put_pixel(x + dx, y + dy, color);
                }
            }
        }
    }
}

impl MediaGenerator for ScreenshotGenerator {
    fn create_output(&self, content: &str, output_path: &Path) -> Result<()> {
        self.generate(content, 80, 24, output_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_screenshot_generation() {
        let config = MediaConfig::default();
        let theme = ThemeConfig::default_theme();
        let generator = ScreenshotGenerator::new(&config, &theme);
        
        let temp_file = NamedTempFile::with_suffix(".png").unwrap();
        let content = "Hello, World!\nThis is a test.";
        
        generator.generate(content, 80, 24, temp_file.path()).unwrap();
        
        assert!(temp_file.path().exists());
    }
}