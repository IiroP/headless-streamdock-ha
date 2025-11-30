# Headless Stream Dock HA controller

This project allows you to use "Stream dock" such as Mirabox N3 connected to (headless) Linux for controlling Home Assistant. The project uses [Mirajazz library](https://github.com/4ndv/mirajazz/) and is partially derived from [OpenDeck Ajazz AKP03 / Mirabox N3 plugin](https://github.com/4ndv/opendeck-akp03).

## Before running the app

To detect the device, you have to set some udev rules. Download the [udev rules](https://github.com/4ndv/opendeck-akp03/blob/main/40-opendeck-akp03.rules) and install them by copying into `/etc/udev/rules.d/` and running `sudo udevadm control --reload-rules`. Unplug and plug again the device after this.

## Configuration

The configuration of this app consists of three things: `.env` file (or other env variables), `config.toml` file and `images/` directory for button images.

`.env` file should contain Home Assistant websocket api url and long-lived access token:

```dotenv
HA_URL="ws://localhost:8123/api/websocket"
HA_TOKEN="verysecrettoken"
```

`config.toml` contains general configuration (brightness, timeout) and button/knob mappings. To get the correct HA values, you can use Home Assistant -> Developer Tools -> Actions and examine the YAML output.

```toml
brightness = 40 # key screen brightness, 0-100
timeout = 30 # timeout in seconds for turning off the key screens

[[buttons]]
id = 0 	# id of the button
domain = "homeassistant" # domain of the command
service = "toggle" # action
entity_id = "light.valot" # entity id
icon = "light.png" # icon for button, corresponding file must be in `images/` directory

[[buttons]]
id = 5
domain = "scene"
service = "turn_on"
entity_id = "scene.dim"
icon = "candle.png"


[[knobs]]
id = 1 # id of the knob
domain = "homeassistant" # domain of the command
service = "turn_on" # action
entity_id = "light.valot" # entity id
key = "brightness_step" # what value is changed
step = 10 # how much the value is changed
```

All images referenced in the config should be placed in the `images/` directory

## Compatibility

This program is only tested with Mirabox N3, but the underlying library supports also similar devices such as Ajazz AKP03. To get parameters for your device, you can examine the `dmesg` output and you should see something like:

```
New USB device found, idVendor=6603, idProduct=1003, bcdDevice= 0.02
```

Which means `vendor_id = 0x6603` and `product_id = 0x1003`.

## Features

- Assign HA actions to buttons
- Assign HA actions to knobs (such as brightness or volume control)
- Set custom pictures for buttons with screens
- Configure timeout for screens
- Configure screen brightness
