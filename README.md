# P2P Failover

A peer-to-peer active-passive failover system written in Rust that manages process execution across multiple nodes based on availability and priority.

## Overview

P2P Failover ensures that a process runs on the highest priority available node, with automatic failover if that node becomes unavailable.

Key features:

- Automatic process management based on node priority
- Real-time node health monitoring
- TCP-based peer communication
- YAML configuration (apologies)

## Installation

1. Clone the repository
2. Build the project:

```bash
cargo build --release
```

## Configuration

Create a `p2p-failover.config.yaml` file in your project directory. Here's an example configuration:

```yaml
ddns:
- name: pc
  ddns: ''
  ip: 127.0.0.1
  port: 8080
  preference: 1
  priority: 100
  last_updated: 2025-02-04 19:19:18 UTC
- name: phone
  ddns: ''
  ip: 100.11.111.111
  port: 8081
  preference: 1
  priority: 20
  last_updated: 2025-01-09 16:45:00 UTC
config_metadata:
  name: pc
  last_updated: 2025-01-11 10:00:00 UTC
execution:
  instructions: ./test-program.sh
  last_updated: 2025-01-11 10:00:00 UTC
```

### Configuration Fields

- `ddns`: List of nodes in the network
  - `name`: Unique identifier for the node
  - `ddns`: Domain name (optional)
  - `ip`: IP address
  - `port`: TCP port for node communication
  - `preference`: Connection preference (0 for DDNS, 1 for IP)
  - `priority`: Node priority (higher number = higher priority)
  - `last_updated`: Timestamp of last update
- `config_metadata`: Node-specific metadata
  - `name`: Name of this node
  - `last_updated`: Configuration timestamp
- `execution`: Process execution settings
  - `instructions`: Command to execute
  - `last_updated`: Last modification timestamp

## Environment Variables

- `P2P_CONFIG_PATH`: Path to config file (default: `p2p-failover.config.yaml`)
- `VERBOSE`: Enable verbose logging (1/true)
- `DEBUG`: Enable debug logging (1/true)

Note: When `DEBUG` is set to `1`, `VERBOSE` is automatically turned on.

## How It Works

1. Each node monitors the health of other nodes in the network through periodic heartbeats
2. The node with the highest priority and availability runs the specified process
3. If a higher priority node becomes available, the process gets killed and started on the other node
4. If the active node fails, the next highest priority available node takes over

## License

MIT
