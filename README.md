# Speculare client

Speculare client is intended to report some 'usefull' data to the Speculare server in order
to manage our iMac more efficiently.

This project is meant to evolve in something more complete and more complexe in a somewhat near future.

## Setup

- install libcpuid dependency
```bash
apt install cpuidtool cpuidtool libcpuid-dev
```
- run (to init .config)
```
/path/to/speculare --config
```
- or copy `configs/exemple.config` into your `$HOME/speculare.config`
```bash
cp configs/exemple.config /etc/speculare.config
```

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.