# PropStream → Pipedrive Integration

Automated real estate lead integration system that pulls qualified property leads from PropStream and syncs them to Pipedrive CRM with intelligent filtering and duplicate detection.

## 🚀 Quick Start

```bash
# Clone the repository
git clone https://github.com/your-username/propstream-pipedrive-integration
cd propstream-pipedrive-integration

# Copy and configure environment variables
cp .env.example .env
# Edit .env with your API credentials

# Copy and configure settings
cp config.example.json config.json
# Edit config.json with your criteria

# Run the application
cargo run
```

The application will start with a monitoring dashboard and begin automated synchronization based on your schedule.

## ✨ Features

### 🔄 Automated Lead Integration
- **PropStream Data Extraction**: Pulls properties from specified geographic regions
- **Intelligent Filtering**: Multi-criteria qualification (equity, ownership, property type)
- **Pipedrive Synchronization**: Automatic lead creation with custom fields
- **Duplicate Detection**: SQLite-based tracking prevents duplicate leads
- **Scheduled Automation**: Cron-based scheduling for hands-free operation

### 🎯 Smart Lead Qualification
- **Geographic Filters**: NJ, PA, DE states with specific county targeting
- **Property Types**: Single family, multi family, vacant land
- **Owner Criteria**: Absentee owners, ownership duration, entity filters
- **Financial Filters**: Equity percentage ranges (30-100%)
- **Timing Filters**: Construction age, ownership duration minimums
- **Lead Scoring**: Automatic prioritization based on multiple factors

### 📊 Monitoring Dashboard
- **Real-Time Statistics**: Track properties fetched, leads qualified, syncs completed
- **Connection Status**: Monitor API health for PropStream, Pipedrive, Database
- **Sync History**: Detailed logs of all integration runs
- **Quality Metrics**: Qualification rates, sync success rates
- **Manual Controls**: Run integration on-demand, test connections

## 🏗️ Architecture

Built with clean, modular architecture following the 5-phase integration process:

```
src/
├── models/              # Data models
│   ├── property.rs     # Property data structures
│   ├── criteria.rs     # Lead qualification criteria
│   ├── lead.rs         # Qualified lead models
│   └── stats.rs        # Integration statistics
├── api/                 # API clients
│   ├── propstream.rs   # PropStream API integration
│   └── pipedrive.rs    # Pipedrive API integration
├── filters/             # Lead qualification
│   ├── qualifier.rs    # Criteria-based filtering
│   └── scorer.rs       # Lead scoring engine
├── db/                  # Database layer
│   ├── repository.rs   # SQLite operations
│   └── schema.sql      # Database schema
├── integration/         # Core integration logic
│   ├── engine.rs       # Main orchestration
│   └── sync.rs         # Scheduled synchronization
├── config/              # Configuration management
│   └── mod.rs          # Config loading/saving
├── dashboard/           # UI dashboard
│   ├── view.rs         # Dashboard interface
│   └── actions.rs      # User actions
└── main.rs             # Application entry point
```

### Technology Stack
- **Rust**: Memory-safe systems programming
- **GPUI**: Native UI framework from Zed
- **SQLx**: Type-safe SQL database operations
- **Reqwest**: HTTP client for API calls
- **Tokio**: Async runtime
- **Tokio-Cron-Scheduler**: Job scheduling
- **Chrono**: Date/time handling
- **Serde**: JSON serialization

## 📋 Integration Process

The 5-phase automated integration process:

### Phase 1: Data Extraction
- Configure PropStream API credentials
- Set geographic parameters (states, counties)
- Define property type filters
- Apply motivated seller criteria

### Phase 2: Lead Qualification
- Apply absentee owner filters
- Check equity ranges (30-100%)
- Validate ownership duration (5+ years)
- Verify construction age (10+ years)
- Score leads for prioritization

### Phase 3: Duplicate Detection
- Check against existing database records
- Verify against Pipedrive leads
- Track by property address + owner combination
- Prevent redundant lead creation

### Phase 4: Pipedrive Synchronization
- Create person records
- Create organization records (for LLCs/Trusts)
- Create deal with custom fields
- Assign to appropriate pipeline/stage

### Phase 5: Quality Control
- Validate data completeness
- Track success/failure rates
- Log detailed sync history
- Update integration statistics

## ⚙️ Configuration

### Environment Variables (.env)
```bash
PROPSTREAM_API_KEY=your_api_key
PIPEDRIVE_API_TOKEN=your_token
DATABASE_URL=sqlite://propstream_pipedrive.db
SCHEDULER_CRON=0 0 9 * * *  # Daily at 9 AM
RUST_LOG=info
```

### Configuration File (config.json)
See `config.example.json` for complete configuration options including:
- API credentials
- Geographic targeting
- Property type filters
- Owner qualification criteria
- Financial thresholds
- Scheduler settings

## 🔧 Development

### Prerequisites
- Rust 1.75+
- macOS (GPUI requirement)
- PropStream API access
- Pipedrive account with API token

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

## 🎯 Lead Criteria (Default Configuration)

### Geographic Targeting
- **States**: NJ, PA, DE
- **Counties**: Camden, Burlington, Gloucester, Atlantic, Cape May, Philadelphia, Delaware, Bucks

### Property Requirements
- **Types**: Single Family, Multi Family, Vacant Land
- **Construction**: 10+ years old
- **Distressed**: Allowed
- **Vacant**: Allowed

### Owner Requirements
- **Absentee**: Required
- **Ownership Duration**: 5+ years
- **Exclude**: LLCs and Trusts
- **Entity Type**: Individual owners preferred

### Financial Requirements
- **Equity Range**: 30% - 100%
- **Property Value**: No minimum/maximum (configurable)

## 🎯 Design Principles

### Clean Architecture
- **Separation of Concerns**: Business logic isolated from UI
- **Type Safety**: Leverages Rust's type system fully
- **Error Handling**: Comprehensive error tracking and logging
- **Async First**: Non-blocking operations throughout

### Data Quality
- **Duplicate Prevention**: Multi-layer duplicate detection
- **Validation**: Data quality checks at every stage
- **Scoring**: Intelligent lead prioritization
- **Tracking**: Complete audit trail in SQLite database

## 🚧 Roadmap

- [ ] Email notifications for integration results
- [ ] Advanced filtering with custom rules
- [ ] Bulk lead import/export
- [ ] Integration with additional CRMs
- [ ] API webhook support
- [ ] Advanced analytics and reporting
- [ ] Multi-user support
- [ ] Team assignment automation

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