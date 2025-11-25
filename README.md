# miam

A minimalist RSS feed reader TUI (Terminal User Interface) built with Rust.

## Features

- **Clean Terminal Interface** - Navigate RSS feeds with vim-like keybindings
- **Multi-Source Support** - Manage multiple RSS/Atom feeds from one place
- **Smart Caching** - Offline access to previously loaded articles
- **Built-in Reader** - Read full articles without leaving the terminal
- **YouTube Support** - RSS feeds for YouTube channels with optional Invidious integration
- **Filter & Search** - Quickly filter feeds and articles in real-time
- **Dual-Line Display** - Clear two-line layout for better readability

## Installation

### Quick Install (Recommended)

Install with a single command:

```bash
curl -sSL https://raw.githubusercontent.com/saravenpi/miam/master/install.sh | bash
```

The installer will:
- Detect your platform automatically (Linux/macOS, x86_64/aarch64)
- Download pre-built binary (if available)
- Or build from source using cargo (requires Rust installed)
- Install to `~/.local/bin/miam`
- Provide instructions for adding to PATH if needed

**Note:** If you don't have Rust installed and pre-built binaries aren't available, install Rust first:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Manual Installation from Source

```bash
git clone https://github.com/saravenpi/miam.git
cd miam
cargo build --release
cp target/release/miam ~/.local/bin/
```

Make sure `~/.local/bin` is in your PATH.

### Adding to PATH

If `~/.local/bin` is not in your PATH, add this to your shell config:

**Bash** (`~/.bashrc` or `~/.bash_profile`):
```bash
export PATH="$PATH:$HOME/.local/bin"
```

**Zsh** (`~/.zshrc`):
```bash
export PATH="$PATH:$HOME/.local/bin"
```

**Fish** (`~/.config/fish/config.fish`):
```fish
set -gx PATH $PATH $HOME/.local/bin
```

### Uninstalling

To uninstall miam:

```bash
rm ~/.local/bin/miam
rm -rf ~/.miam      # Remove cache (optional)
rm ~/.miam.yml      # Remove config (optional)
```

## Quick Start

1. Launch miam:
```bash
miam
```

2. Press `a` to add your first RSS feed
3. Enter the feed URL and press Enter
4. Press `r` to refresh and load the feed
5. Use `Tab` to switch between feeds and articles
6. Press `o` to open an article

## Commands

### Check Version

```bash
miam --version
```

### Upgrade to Latest Version

Miam includes a built-in self-update mechanism that downloads the latest release from GitHub:

```bash
miam upgrade
```

This will:
- Check for the latest release on GitHub
- Download and install the new version if available
- Show a progress bar during download
- Replace the current binary with the new version

## Configuration

miam uses a YAML configuration file located at `~/.miam.yml`.

### Example Configuration

```yaml
feeds:
  Hacker News: https://news.ycombinator.com/rss
  Rust Blog: https://blog.rust-lang.org/feed.xml
  The Verge: https://www.theverge.com/rss/index.xml
  Tech Crunch: https://techcrunch.com/feed/
  Ars Technica: https://feeds.arstechnica.com/arstechnica/index

settings:
  invidious: false
  invidious_instance: yewtu.be
```

### Configuration Options

#### Feeds Section

The `feeds` section is a key-value map where:
- **Key**: Display name for the feed
- **Value**: RSS/Atom feed URL

#### Settings Section

- `invidious` (boolean, default: `false`)
  - When enabled, YouTube links open through an Invidious instance instead of YouTube.com
  - Useful for privacy-conscious users

- `invidious_instance` (string, default: `yewtu.be`)
  - Specifies which Invidious instance to use
  - Available instances: `yewtu.be`, `vid.puffyan.us`, `invidious.flokinet.to`, `invidious.privacydev.net`, `iv.melmac.space`

## Keybindings

### Global

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Switch focus between Feeds and Items panels |
| `q` | Quit the application |
| `Ctrl+C` | Force quit |

### Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `g g` | Go to top (press `g` twice) |
| `G` | Go to bottom |

### Feed Management

| Key | Action |
|-----|--------|
| `a` | Add new feed |
| `d` | Delete selected feed |
| `r` | Refresh all feeds |
| `Enter` | Load selected feed (when in Feeds panel) |

### Article Actions

| Key | Action |
|-----|--------|
| `Enter` / `o` | Open article in browser |
| `o` | Open article in reader (non-YouTube articles) |
| `/` | Filter articles by title |

### Filter Mode

| Key | Action |
|-----|--------|
| `/` | Activate filter (in Feeds or Items panel) |
| `Type` | Filter results in real-time |
| `Backspace` | Remove characters from filter |
| `Enter` / `Esc` | Exit filter mode |

### Reader Mode

| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| `Space` / `Page Down` | Scroll down one page |
| `b` / `Page Up` | Scroll up one page |
| `g` | Jump to top |
| `G` | Jump to bottom |
| `o` | Open article in browser |
| `Esc` / `q` | Exit reader mode |

## Usage Examples

### Adding RSS Feeds

1. **From the UI:**
   - Press `a`
   - Enter the feed URL
   - Press `Enter`
   - The feed is automatically saved to `~/.miam.yml`

2. **Manually editing config:**
   - Edit `~/.miam.yml`
   - Add feeds in the format: `Feed Name: https://feed.url/rss`
   - Save and restart miam

### YouTube RSS Feeds

YouTube channels have RSS feeds! Format:
```
https://www.youtube.com/feeds/videos.xml?channel_id=CHANNEL_ID
```

Example:
```yaml
feeds:
  Fireship: https://www.youtube.com/feeds/videos.xml?channel_id=UCsBjURrPoezykLs9EqgamOA
```

### Filtering

Press `/` to filter feeds or articles:
- In **Feeds panel**: Filters by feed name
- In **Items panel**: Filters by article title
- Case-insensitive substring matching
- Real-time results as you type

Example: Type "rust" to see all Rust-related articles

## Cache

miam caches feed articles in `~/.miam/` for offline access.

- Each feed has its own cache file
- Cache loads instantly on startup
- Updates in background when refreshing
- Deduplicates articles automatically

Cache files are named based on the feed name (sanitized):
```
~/.miam/Hacker_News.yml
~/.miam/Rust_Blog.yml
```

## Feed Limit Notes

- **YouTube feeds**: Limited to 15 most recent videos (YouTube RSS limitation)
- **Regular feeds**: Depends on the feed provider (typically 10-50 items)

A note appears in the status bar when YouTube feeds are present.

## Tips & Tricks

1. **Quick Refresh**: Press `r` from any panel to refresh all feeds

2. **Focus Management**: The focused panel has a colored border (blue/purple)

3. **"All Feeds" View**: Select the `★ All` option to see articles from all feeds combined

4. **Reading Articles**:
   - YouTube videos automatically open in browser
   - Regular articles can be read in-app (press `o`) or in browser (press `o` again when YouTube or `Enter`)

5. **Filter Performance**: Filtering is case-insensitive and works on substrings

6. **Background Updates**: When feeds refresh, you'll see a spinner. The UI remains responsive.

## Troubleshooting

### Feed won't load
- Check the URL is valid RSS/Atom feed
- Some feeds require specific User-Agent headers (not currently supported)
- Try the feed URL in a browser first

### Articles not showing
- Press `r` to refresh
- Check internet connection
- Some feeds may be temporarily unavailable

### Cache issues
- Clear cache: `rm -rf ~/.miam/`
- Restart miam and press `r`

## Building from Source

Requirements:
- Rust 1.70 or higher
- Cargo

```bash
git clone https://github.com/saravenpi/miam.git
cd miam
cargo build --release
```

The binary will be in `target/release/miam`.

## License

This project is licensed under the MIT License.

## Credits

Built by [saravenpi](https://github.com/saravenpi)

Powered by:
- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) - Terminal manipulation
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP client
- [rss](https://github.com/rust-syndication/rss) - RSS parser
- [atom_syndication](https://github.com/rust-syndication/atom) - Atom parser
- [readability](https://github.com/kumabook/readability) - Article extraction
