# Yagami

A simple web page to control a single light.

`yagami` uses the `homeassistant` REST API to get the state of the light (on or off), and to toggle the state when the web page is clicked.

<div align="center">
  <img src="images/on.png" alt="Screenshot of the interface when the light is on" width="300"/>
  <img src="images/off.png" alt="Screenshot of the interface when the light is off" width="300"/>
</div>

## Environment variables

The following env vars need to be set:

- `LIGHT_ID`=`<home assistant light ID, eg: light.foobar>`
- `YAGAMI_TOKEN`=`<home assistant API token>`
- `YAGAMI_PUBLIC_PATH`=`<path to the directory that contains html file and images>`
- `HOME_ASSISTANT_URL`=`<base URL of home assistant server, eg: 192.168.1.1:8123>`

## Building from source

### Prerequisites

- Rust (https://www.rust-lang.org/tools/install)
- Cargo (comes with Rust)

### Build instructions

1. Clone this repository
2. Run `cargo build --release`
3. The binary will be in the `target/release` directory.

## Running

You can run `yagami` locally:

```bash
cargo run --release
```

## Systemd service

A systemd unit file, `yagami.service`, is provided to run yagami as a service.

To use it:

- Copy the binary `yagami` to `/usr/bin/` (see Build instructions)
- Create an environment file `/var/lib/yagami/yagami.env` that contains the necessary env variables (see Environment variables)
- Copy the file `yagami.service` to `/etc/systemd/system/` and enable it with:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now yagami.service
```
