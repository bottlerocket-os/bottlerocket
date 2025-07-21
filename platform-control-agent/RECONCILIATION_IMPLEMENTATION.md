# Reconciliation Loop Implementation

## Overview
The reconciliation loop has been successfully implemented to provide automatic configuration drift detection and correction for the Platform Control Agent.

## Architecture

### Components

1. **ConfigReconciler** (`src/reconciler/mod.rs`)
   - Main reconciliation engine
   - Runs periodic checks comparing desired vs actual configuration
   - Handles drift detection and automatic correction
   - Integrates with event system for observability

2. **ReconcilerConfig** (`src/reconciler/config.rs`)
   - Configuration management for reconciliation behavior
   - Environment variable support for easy deployment configuration
   - Configurable intervals, thresholds, and ignored fields

3. **ConfigDiff** (`src/reconciler/diff.rs`)
   - Comprehensive configuration comparison logic
   - Field-by-field drift detection
   - Severity classification (Info, Warning, Critical)
   - Support for all Bottlerocket settings types

## Features

### Configuration Options
- **RECONCILER_ENABLED**: Enable/disable reconciliation (default: true)
- **RECONCILER_INTERVAL**: Check interval in seconds (default: 300)
- **RECONCILER_AUTO_CORRECT**: Enable automatic correction (default: true)

### Drift Detection
- Compares desired configuration (from StateManager) with actual Bottlerocket settings
- Detects differences in:
  - Kubernetes settings (API server, certificates, DNS)
  - Network settings (hostname)
  - Kernel settings (lockdown, sysctl)
  - Host containers configuration
  - NTP settings

### Drift Correction
- Automatic correction based on severity thresholds
- Critical fields always trigger correction
- Configurable correction behavior
- Event emission for all actions

### Field Management
- **Ignored Fields**: Fields that change frequently or are externally managed
  - motd
  - host_containers.admin.user_data
- **Critical Fields**: Fields that must always match
  - kubernetes.api_server
  - kubernetes.cluster_certificate
  - network.hostname

## Events

The reconciliation system emits the following events:
- **ReconciliationStarted**: When the loop begins
- **ReconciliationCompleted**: After each check cycle
- **ReconciliationFailed**: On errors
- **ConfigurationDriftDetected**: When drift is found
- **ConfigurationDriftCorrected**: When drift is automatically fixed

## Integration

The reconciler is integrated into the main service lifecycle:
- Started automatically when the service starts
- Runs in a background task
- Gracefully shuts down on service termination
- Shares state with the main gRPC service

## Testing

A comprehensive test script is provided at `test/test_reconciler.sh` that validates:
- Basic reconciliation functionality
- Drift detection after configuration reset
- Automatic correction behavior
- Reconciliation disable functionality

## Future Enhancements

1. **Metrics Export**
   - Prometheus metrics for drift detection rate
   - Correction success/failure counts
   - Reconciliation loop timing

2. **Advanced Policies**
   - Time-based correction windows
   - Approval-based corrections for critical changes
   - Multi-level correction strategies

3. **Webhook Integration**
   - External notifications on drift detection
   - Integration with monitoring systems
   - Slack/PagerDuty alerts

4. **Enhanced Comparison**
   - Deep diff with JSON patch generation
   - Semantic diff for complex structures
   - Historical drift tracking