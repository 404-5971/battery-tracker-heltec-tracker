# Battery Tracker ESP32-S3

## Setup

To ensure you have the exact same Rust toolchain version used in this project, run:

```bash
espup install --toolchain-version 1.90.0.0 --name esp
```

## Monitoring Output

For watching device output, I'm using [tio](https://aur.archlinux.org/packages/tio). It isn't that great and I'll likely be switching tools in the future, but it is useful for staying connected and reconnecting automatically between device resets.

To determine which serial port your device is connected to, run:

```bash
tio --list
```

Once you've identified the correct port (e.g., `/dev/ttyACM0` or `/dev/ttyUSB0`), use it to monitor the output:

```bash
tio <device_port>
```
