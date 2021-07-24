<div align="center">
  <h1>Speculare Client</h1>
  <p>
    <strong>Metrics for your servers</strong>
  </p>
  <p>

[![Apache 2 License](https://img.shields.io/badge/license-Apache%202-blue.svg)](LICENSE)
[![CI](https://github.com/Martichou/speculare-client/workflows/CI/badge.svg)](https://github.com/Martichou/speculare-client/actions)
[![Docs](https://img.shields.io/badge/Docs-latest-green.svg)](https://docs.speculare.cloud)

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

Download
--------------------------

You can find latest versioned archives [here](https://github.com/Martichou/speculare-client/releases), with binaries for all platforms.

You can also find debug build in the CI'artifacts [here](https://github.com/speculare-cloud/speculare-client/actions/workflows/ci.yml).

### Configurations
Speculare Client need a config file (see Example.toml for example). You can save it anywhere, you just have to specify it's path to speculare-client when launching it.
```bash
$ speculare-client "/path/to/Config.toml"
```

Contributing
--------------------------

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.