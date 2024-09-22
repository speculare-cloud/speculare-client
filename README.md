<div align="center">
  <h1>Speculare Client</h1>
  <p>
    <strong>Metrics for your servers</strong>
  </p>
  <p>

[![Apache 2 License](https://img.shields.io/badge/license-Apache%202-blue.svg)](LICENSE)
[![CI](https://github.com/speculare-cloud/speculare-client/actions/workflows/ci.yml/badge.svg)](https://github.com/speculare-cloud/speculare-client/actions)

  </p>
</div>

Speculare client (SP from now on) real-time monitoring Agent collects a lot of metrics from systems, hardware, VM, and applications with the least configuration possible. It runs permanently on your servers, computers, etc.

You can install SP on almost every platform (Linux, macOS, Windows).
As things stand, SP is still in heavy development and is expected to become much more complex and complete over time.

Dev setup
--------------------------

- Install all deps
```bash
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
$ sudo apt-get install libssl-dev libpq-dev pkg-config build-essential
```

Configurations
--------------------------

Speculare Client need a config file (see client.example.config for example). You can save it anywhere, 
you just have to specify it's path to speculare-client when launching it.

By by default speculare-client will
try to open it at `/etc/speculare/client.config` if it's not defined in the CLI.
```bash
$ speculare-client -c "/path/to/client.config"
```

Contributing
--------------------------

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.
