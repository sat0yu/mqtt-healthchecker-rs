# mqtt-healthchecker-rs

```
USAGE:
    mqtt-healthchecker-rs [FLAGS] --host <HOST> --request <REQUEST_TOPIC> --response <RESPONSE_TOPIC>

FLAGS:
    -e, --expect      The expected payload in the healthckeck response
        --help        Prints help information
    -i, --interval    The interval period for sending a request (default: 2 seconds)
    -p, --payload     The payload for the healthckeck request (default: "healthcheck")
    -t, --timeout     The timeout (seconds) to exit with an error status (default: 16 seconds)
    -V, --version     Prints version information

OPTIONS:
    -h, --host <HOST>                  Sets the MQTT broker host
        --request <REQUEST_TOPIC>      Sets the topic name to which sends requests
        --response <RESPONSE_TOPIC>    Sets the topic name to which the response is sent
```
