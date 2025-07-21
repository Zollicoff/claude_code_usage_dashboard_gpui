# Claude Code Usage Dashboard

A blazing-fast, native analytics dashboard for Claude Code usage built with pure Rust and GPUI.

![Claude Code Usage Dashboard](src/images/Screenshot%202025-07-21%20at%2015.42.34.png)

## 🚀 Quick Start

```bash
# Clone and run
git clone https://github.com/your-username/claude_usage_dashboard_gpui
cd claude_usage_dashboard_gpui
cargo run
```

The dashboard will automatically discover and analyze usage data from your local data directory.

## ✨ Features

### 📊 Real-Time Analytics
- **Accurate Cost Tracking**: Precise pricing calculations for all models
- **Token Analysis**: Input, output, cache read/write breakdowns  
- **Project Insights**: Resource consumption per project
- **Session History**: Detailed interaction tracking
- **Smart Deduplication**: Prevents double-counting of usage entries

### ⚡ Performance First
- **Instant Startup**: Loads recent data immediately, processes full dataset in background
- **Native Speed**: Pure Rust with GPUI - no web technologies
- **Efficient Processing**: Handles hundreds of data files smoothly
- **Zero Lag**: Instant tab switching and UI updates
- **Time Range Filtering**: Lightning-fast time range switching (All Time, Last 30 Days, Last 7 Days)

### 🎯 Dashboard Views
Navigate with keyboard shortcuts or mouse clicks:
- **Tab Navigation**: Press 1-5 to switch between views
- **Time Range Filters**: Alt+1 (All Time), Alt+2 (Last 30 Days), Alt+3 (Last 7 Days)
- **Overview**: Key metrics and summary cards
- **Models**: Usage breakdown by AI model with scrollable detailed lists
- **Projects**: Project-wise resource analysis with scrollable project cards
- **Sessions**: Individual session tracking with scrollable timeline
- **Timeline**: Visual usage trends with scrollable daily usage patterns

All views feature smooth scrolling for content that exceeds the viewport, ensuring easy navigation through large datasets.

## 🏗️ Architecture

Built from the ground up with clean, modular architecture:

```
src/
├── analytics/           # Data processing engine
│   ├── models.rs       # Clean data structures
│   ├── processor.rs    # Data file processing
│   ├── calculator.rs   # Accurate pricing calculations
│   └── aggregator.rs   # Data grouping and analysis
├── app/
│   ├── views/          # UI views and components
│   ├── models/         # Application state
│   └── actions.rs      # User interactions
├── theme/
│   ├── colors.rs       # Color system (light/dark themes)
│   ├── registry.rs     # Theme state management
│   └── settings.rs     # Theme persistence
├── ui/
│   └── formatting.rs   # Display utilities
└── main.rs            # Application entry point
```

### Technology Stack
- **Rust**: Memory-safe systems programming
- **GPUI**: Native UI framework from Zed
- **Chrono**: Date/time handling
- **Serde**: JSON serialization
- **WalkDir**: Efficient file system traversal

## 📈 Performance Metrics

- **Startup**: < 100ms to interactive UI
- **Processing**: 12,000+ entries processed in seconds
- **Memory**: Minimal footprint (~50MB)
- **Rendering**: 60fps smooth scrolling and transitions

## 🔧 Development

### Prerequisites
- Rust 1.75+
- macOS (GPUI requirement)
- Usage data in supported format

### Build Commands
```bash
# Development build
cargo run

# Optimized release
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

### Data Processing Pipeline
1. **Discovers data files** in the configured directory
2. **Parses entries** with intelligent deduplication
3. **Calculates costs** using accurate pricing models
4. **Filters noise** - removes zero-token entries
5. **Aggregates metrics** by model, project, and time

## 🎯 Design Principles

### Clean Architecture
- **Separation of Concerns**: Business logic isolated from UI
- **Pure Rust**: No external runtime dependencies
- **Type Safety**: Leverages Rust's type system fully
- **GPUI Native**: Built for maximum framework efficiency

### Performance Optimizations
- **Smart Loading**: Progressive data loading strategy
- **Efficient Aggregation**: Single-pass data processing
- **Memory Efficiency**: Zero-copy where possible
- **Native Rendering**: GPU-accelerated UI
- **Smooth Scrolling**: GPUI-native overflow handling for large datasets

## 🚧 Roadmap

- [ ] Real-time data refresh
- [ ] Advanced filtering options
- [ ] Data export functionality
- [ ] Custom date ranges
- [ ] Trend analysis
- [ ] Performance metrics

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style
- Follow Rust standard conventions
- Use `cargo fmt` before committing
- Add tests for new functionality
- Keep commits focused and atomic

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Zed Team** for the incredible GPUI framework
- **Rust Community** for excellent tooling and libraries
- **Contributors** who help improve this project