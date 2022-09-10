# mqtt-healthchecker-rs

```
USAGE:
    mqtt-healthchecker-rs [OPTIONS] --host <HOST> --request <REQUEST_TOPIC> --response <RESPONSE_TOPIC>

OPTIONS:
    -e, --expect                       The expected payload in the healthckeck response
    -h, --host <HOST>                  Sets the MQTT broker host [env: MQTT_HEALTHCHECKER_HOST=]
        --help                         Print help information
    -i, --interval                     The interval period for sending a request (default: 2
                                       seconds)
    -p, --payload                      The payload for the healthckeck request (default:
                                       "healthcheck")
        --request <REQUEST_TOPIC>      Sets the topic name to which sends requests [env:
                                       MQTT_HEALTHCHECKER_REQUEST_TOPIC=]
        --response <RESPONSE_TOPIC>    Sets the topic name to which the response is sent [env:
                                       MQTT_HEALTHCHECKER_RESPONSE_TOPIC=]
    -t, --timeout                      The timeout (seconds) to exit with an error status (default:
                                       16 seconds)
    -V, --version                      Print version information
```
