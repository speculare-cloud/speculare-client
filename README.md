Speculare client
========
![CI](https://img.shields.io/github/workflow/status/Martichou/speculare-client/Rust)
[![AGPL License](https://img.shields.io/badge/license-AGPL-blue.svg)](LICENSE)
![macOS](https://github.com/Martichou/speculare-client/workflows/macOS/badge.svg)
![Linux](https://github.com/Martichou/speculare-client/workflows/Linux/badge.svg)

Speculare client (SP from now on) real-time monitoring Agent collects a lot of metrics from systems, hardware, VM, and applications with the least configuration possible. It runs permanently on your servers, computers, etc.

You can install SP on almost every platform (Linux, macOS, Windows).
As things stand, SP is still in heavy development and is expected to become much more complex and complete over time.

Download
--------------------------

You can find lastest versioned archives [here](https://github.com/Martichou/speculare-client/releases), with binaries for all platforms.

### Configurations
```bash
➜  ~ speculare-client --config
```
<img src="assets/speculare_config.svg" width="100%">

Configuring after checkout (dev)
--------------------------

- run speculare and create the config file
```bash
➜  ~ cargo run --config
```

Contributing
--------------------------

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.