# PropStream → Pipedrive Integration Setup Guide

## Overview

This application automates the process of extracting qualified property leads from PropStream and syncing them to your Pipedrive CRM with intelligent filtering and duplicate detection.

## Prerequisites

### System Requirements
- **Operating System**: macOS (required for GPUI framework)
- **Rust**: Version 1.75 or higher
- **Memory**: Minimum 4GB RAM recommended
- **Disk Space**: 500MB for dependencies and database

### API Access Requirements
1. **PropStream Account** with API access
   - API Key from PropStream dashboard
   - Active subscription with API permissions

2. **Pipedrive Account**
   - API Token from Settings → API
   - Permissions to create deals, persons, and organizations

## Installation Steps

### 1. Install Rust

If you don't have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup update
```

### 2. Clone the Repository

```bash
git clone https://github.com/your-username/propstream-pipedrive-integration
cd propstream-pipedrive-integration
```

### 3. Configure Environment Variables

Copy the example environment file:

```bash
cp .env.example .env
```

Edit `.env` with your actual credentials:

```bash
PROPSTREAM_API_KEY=your_actual_propstream_api_key
PIPEDRIVE_API_TOKEN=your_actual_pipedrive_api_token
DATABASE_URL=sqlite://propstream_pipedrive.db
SCHEDULER_CRON=0 0 9 * * *
RUST_LOG=info
```

### 4. Configure Application Settings

Copy the example configuration:

```bash
cp config.example.json config.json
```

Edit `config.json` to customize:
- Geographic targeting (states, counties)
- Property type preferences
- Owner qualification criteria
- Financial thresholds
- Scheduler settings

### 5. Build and Run

```bash
# Build the application
cargo build --release

# Run the application
cargo run --release
```

## Configuration Guide

### Geographic Filters

Target specific areas by configuring:

```json
"geographic": {
  "states": ["NJ", "PA", "DE"],
  "counties": ["Camden", "Burlington", "Gloucester"],
  "exclude_cities": ["City Name"]
}
```

### Property Type Filters

Choose which property types to include:

```json
"property_types": {
  "included_types": ["SingleFamily", "MultiFamily", "VacantLand"],
  "allow_distressed": true,
  "allow_vacant": true
}
```

Options:
- `SingleFamily` - Single family residences
- `MultiFamily` - Multi-unit properties
- `VacantLand` - Land parcels
- `Commercial` - Commercial properties

### Owner Criteria

Filter by owner characteristics:

```json
"owner": {
  "require_absentee": true,      // Only absentee owners
  "exclude_llc": true,            // Exclude LLC ownership
  "exclude_trust": true,          // Exclude trust ownership
  "min_ownership_years": 5.0      // Minimum years of ownership
}
```

### Financial Filters

Set equity and value thresholds:

```json
"financial": {
  "min_equity_percent": 30.0,     // Minimum 30% equity
  "max_equity_percent": 100.0,    // Maximum 100% equity
  "min_property_value": 50000,    // Optional: minimum value
  "max_property_value": 1000000   // Optional: maximum value
}
```

### Timing Filters

Control property age requirements:

```json
"timing": {
  "min_construction_years": 10.0,        // Built 10+ years ago
  "max_days_since_last_sale": null       // Optional: recent sales filter
}
```

### Scheduler Configuration

Control when the integration runs:

```json
"scheduler": {
  "enabled": true,
  "cron_expression": "0 0 9 * * *",      // Daily at 9 AM
  "max_leads_per_run": 100               // Process up to 100 leads
}
```

**Cron Expression Examples:**
- `0 0 9 * * *` - Daily at 9:00 AM
- `0 0 */6 * * *` - Every 6 hours
- `0 0 9 * * MON` - Every Monday at 9:00 AM
- `0 30 8 * * MON-FRI` - Weekdays at 8:30 AM

## First Run

### 1. Test API Connections

When you first launch the application:

1. Click "Test Connections" to verify:
   - PropStream API credentials
   - Pipedrive API credentials
   - Database connectivity

2. All three should show "Connected" status

### 2. Manual Test Run

Before enabling the scheduler:

1. Click "Run Integration" to perform a manual test
2. Monitor the statistics dashboard for:
   - Properties fetched from PropStream
   - Leads qualified by criteria
   - Leads synced to Pipedrive
   - Duplicates skipped

### 3. Review Results

Check your Pipedrive account to verify:
- New leads appear in the correct pipeline
- Custom fields are populated correctly
- Person/organization records are created

## Monitoring

### Dashboard Metrics

The dashboard displays real-time metrics:

- **Properties Fetched**: Total properties pulled from PropStream
- **Qualified Leads**: Properties meeting your criteria
- **Synced to Pipedrive**: Successfully created leads
- **Duplicates Skipped**: Prevented duplicate leads

### Connection Status

Monitor the health of:
- PropStream API connection
- Pipedrive API connection
- Database connection

### Database Location

The SQLite database stores:
- Lead history and tracking
- Duplicate detection records
- Sync history and statistics

Default location: `./propstream_pipedrive.db`

## Troubleshooting

### PropStream API Issues

**Error: Invalid API Key**
- Verify your API key in `.env`
- Check PropStream dashboard for API permissions

**Error: No properties returned**
- Verify your geographic filters
- Check if properties exist matching your criteria
- Confirm API rate limits aren't exceeded

### Pipedrive API Issues

**Error: Invalid API Token**
- Verify your token in `.env`
- Regenerate token in Pipedrive settings if needed

**Error: Failed to create lead**
- Check Pipedrive API permissions
- Verify custom field configuration
- Review error messages in logs

### Database Issues

**Error: Cannot open database**
- Ensure write permissions in application directory
- Check disk space availability

**Duplicate detection not working**
- Database may be corrupted
- Delete `propstream_pipedrive.db` to reset (loses history)

### Build Issues

**GPUI framework errors**
- Ensure you're on macOS
- Update to latest macOS version
- Install Xcode command line tools: `xcode-select --install`

**Cargo build failures**
- Update Rust: `rustup update`
- Clean build: `cargo clean && cargo build`
- Check internet connectivity for dependencies

## Advanced Configuration

### Custom Field Mapping

To map additional PropStream fields to Pipedrive:

1. Edit `src/models/lead.rs`
2. Add fields to `PipedriveCustomFields`
3. Update the conversion in `From<&QualifiedLead> for PipedriveLead`

### Lead Scoring Adjustments

Modify lead prioritization in `src/filters/scorer.rs`:

- Adjust score weights for equity, ownership, etc.
- Add new scoring factors
- Change score tier thresholds

### Notification Setup

To add email notifications:

1. Add `lettre` crate to dependencies
2. Create notification module
3. Hook into integration run results

## Performance Tuning

### Optimize Batch Size

Adjust `max_leads_per_run` in config:
- Lower values: More frequent, smaller batches
- Higher values: Less frequent, larger batches

### Database Performance

For large lead volumes:
- Regularly vacuum database: `sqlite3 propstream_pipedrive.db "VACUUM;"`
- Consider moving to PostgreSQL for production use

### API Rate Limiting

Respect API limits:
- PropStream: Check your plan's rate limits
- Pipedrive: Typically 100 requests per 10 seconds

## Security Best Practices

1. **Never commit `.env` to git**
   - Already in `.gitignore`
   - Use environment-specific files

2. **Protect API credentials**
   - Store securely
   - Rotate regularly
   - Use environment variables in production

3. **Database security**
   - Restrict file permissions
   - Regular backups
   - Encrypt if storing sensitive data

4. **Access control**
   - Limit who can run the application
   - Monitor sync history for anomalies

## Support and Maintenance

### Logs

Application logs provide detailed information:
- Set `RUST_LOG=debug` for verbose logging
- Set `RUST_LOG=warn` for errors only
- Logs output to console by default

### Backup

Regular backups recommended for:
- SQLite database (`propstream_pipedrive.db`)
- Configuration file (`config.json`)
- Environment variables (`.env`)

### Updates

Keep the application updated:
```bash
git pull origin main
cargo build --release
```

## Getting Help

If you encounter issues:

1. Check logs for detailed error messages
2. Review troubleshooting section above
3. Verify API credentials and permissions
4. Test with smaller data sets first
5. Open an issue on GitHub with:
   - Error messages
   - Configuration (without credentials)
   - Steps to reproduce

## License

MIT License - See LICENSE file for details
