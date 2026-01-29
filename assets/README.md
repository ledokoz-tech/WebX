# WebX Browser Assets

Collection of visual and audio assets for the WebX browser.

## Asset Categories

### Icons (`assets/icons/`)
Application icons, toolbar icons, and interface symbols.

#### Application Icons
- `app-icon-16.png` - 16x16 application icon
- `app-icon-32.png` - 32x32 application icon
- `app-icon-48.png` - 48x48 application icon
- `app-icon-128.png` - 128x128 application icon
- `app-icon-256.png` - 256x256 application icon
- `app-icon.svg` - Vector application icon

#### Toolbar Icons
- `new-tab.svg` - New tab button
- `close-tab.svg` - Close tab button
- `back.svg` - Back navigation
- `forward.svg` - Forward navigation
- `refresh.svg` - Refresh page
- `home.svg` - Home button
- `bookmarks.svg` - Bookmarks
- `downloads.svg` - Downloads
- `settings.svg` - Settings
- `extensions.svg` - Extensions

#### Status Icons
- `loading-spinner.gif` - Loading animation
- `secure-connection.svg` - HTTPS secure indicator
- `warning.svg` - Security warning
- `error.svg` - Error indicator
- `info.svg` - Information icon

### Fonts (`assets/fonts/`)
Custom fonts and typography assets.

#### Interface Fonts
- `Inter-Regular.ttf` - Primary interface font
- `Inter-Bold.ttf` - Bold interface font
- `Inter-Italic.ttf` - Italic interface font
- `FiraCode-Regular.ttf` - Monospace font for developer tools

#### Font Licensing
All fonts are open-source and properly licensed for distribution.

### Images (`assets/images/`)
Background images, illustrations, and promotional graphics.

#### UI Backgrounds
- `dark-theme-bg.png` - Dark theme background pattern
- `light-theme-bg.png` - Light theme background pattern
- `startup-splash.png` - Application startup screen

#### Promotional Assets
- `banner-1200x630.png` - Social media banner
- `screenshot-browser.png` - Browser interface screenshot
- `logo-horizontal.svg` - Horizontal logo variant
- `logo-vertical.svg` - Vertical logo variant

### Sounds (`assets/sounds/`)
Audio feedback and notification sounds.

#### Interface Sounds
- `notification.wav` - General notification sound
- `download-complete.wav` - Download completion sound
- `error-alert.wav` - Error notification sound
- `tab-created.wav` - New tab creation sound
- `tab-closed.wav` - Tab closure sound

#### Sound Specifications
- Format: WAV (uncompressed for quality)
- Sample Rate: 44.1 kHz
- Bit Depth: 16-bit
- Duration: 1-3 seconds maximum

### Themes (`assets/themes/`)
Custom theme packages and color schemes.

#### Color Schemes
- `dark-theme.json` - Dark color palette
- `light-theme.json` - Light color palette
- `high-contrast.json` - High contrast accessibility theme
- `solarized-dark.json` - Solarized dark variant
- `solarized-light.json` - Solarized light variant

#### Theme Structure
```json
{
  "name": "Dark Theme",
  "author": "WebX Team",
  "version": "1.0.0",
  "colors": {
    "background-primary": "#121212",
    "background-secondary": "#1e1e1e",
    "text-primary": "#e0e0e0",
    "accent-primary": "#4d90fe"
  },
  "typography": {
    "font-family": "Inter",
    "font-size-base": "14px",
    "line-height": "1.5"
  }
}
```

## Asset Guidelines

### File Naming Convention
- Use kebab-case for file names
- Include size/resolution in filenames when relevant
- Use descriptive names that indicate purpose
- Version files when making updates

### Quality Standards
- **Icons**: SVG preferred, PNG fallbacks provided
- **Images**: PNG for lossless, JPEG for photographs
- **Fonts**: WOFF2 format preferred for web use
- **Sounds**: WAV for quality, MP3 for distribution

### Size Optimization
- Icons: Keep under 10KB each
- Images: Compress appropriately for web use
- Fonts: Subset to required characters when possible
- Sounds: Trim silence and normalize volume

### Accessibility Considerations
- Provide alternative text for all images
- Ensure sufficient color contrast ratios
- Include high-contrast theme variants
- Support reduced motion preferences

## Asset Creation Process

### Design Workflow
1. Create vector originals in design software
2. Export to required formats and sizes
3. Optimize files for web delivery
4. Test across different platforms and resolutions
5. Document usage guidelines

### Version Control
- Original design files stored separately
- Production assets committed to repository
- Major revisions tagged with version numbers
- Changelog maintained for asset updates

## Usage Instructions

### In Application Code
```rust
// Loading an icon
let icon_path = "assets/icons/new-tab.svg";
let icon_data = std::fs::read(icon_path)?;

// Using a theme
let theme_path = "assets/themes/dark-theme.json";
let theme: ThemeConfig = serde_json::from_str(&std::fs::read_to_string(theme_path)?)?;
```

### Build Integration
Assets are automatically bundled during the build process:
- Icons are embedded in binary for performance
- Fonts are packaged with application
- Themes are loaded at runtime
- Sounds are included in distribution

## Contributing Assets

### Submission Guidelines
1. Follow established naming conventions
2. Provide multiple sizes/formats when applicable
3. Include usage documentation
4. Ensure proper licensing and attribution
5. Test across supported platforms

### Review Process
- Design consistency check
- Technical quality verification
- Accessibility compliance review
- Performance impact assessment
- Cross-platform compatibility testing

## Asset Updates and Maintenance

### Regular Maintenance Tasks
- Update outdated assets quarterly
- Review and optimize file sizes
- Test compatibility with new platforms
- Refresh promotional materials
- Update documentation

### Deprecation Policy
- Maintain backward compatibility for 2 major versions
- Clearly mark deprecated assets
- Provide migration guidance
- Remove deprecated assets after grace period

## Licensing

All assets are licensed under the same terms as the WebX project (GPL-3.0).
Third-party assets are properly attributed and used within license terms.