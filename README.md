# Yagami

A simple web page to control a single light.

`yagami` uses the `homeassistant` REST API to get the state
of the light (on or off), and to toggle the state when the web
page is clicked.

## Env vars

`LIGHT_ID`=<home assistant light ID. Eg: light.foobar>
`YAGAMI_TOKEN`=<home assistant API token>
`YAGAMI_PUBLIC_PATH`=<path to the directory that contains html file and images>
