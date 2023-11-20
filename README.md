# hqcli

A very hacky CLI tool to log worktime in HelloHQ.

## Usage

Submit working time for today from given start and end time.
```
hqcli --start 9:00 --end 16:00
```

You can also specify a date range.
```
hqcli --start '12.12.2023 9:00' --end '12.12.2023 16:00'
```

You can also pass only a start time and work duration.
```
hqcli --start 9:00 --time 8h30m
```

If you have a default `startTime` defined, you can also omit the start time.
```
hqcli --time 8h30m
```

You can also pass a pause duration and you work duration is calculated based on that.
```
hqcli --start 9:00 --end 16:00 --pause 45m
```

When the end time is not passed, the current time will be used for that.
```
hqcli --start 9:00 --pause 45m
```

And if you have both your default `startTime` and `pauseDuration` set, you only need to execute the CLI with no parameters at the end of your workday.
```
hqcli
```

For more information, please use the help manual.
```
hqcli help
```

## Config

```yaml
endpoint: "https://<company>.hellohq.io"
# Session tokens can be exctracted from local storage
# after logging in. 
session:
  # The local storage key - mostly a 6 digit number.
  key: "598280"
  # The local storage value (the session secret).
  value: "aGVsbG8gd29ybG..."
# Some optional default values.
defaults:
  # Default start work time. Will be used when no
  # `--start` has been specified.
  startTime: "08:00"
  # Default pause duration. Will be used when no
  # `--pause` has been specified.
  pause: "45m"
  # Defaults for the `csv` sub command.
  csv:
    # Format used for the csv entries.
    # See `hqcli csv --help` for more information.
    format: "date:%d.%m.%Y,start_time:%H:%M,end_time:%H:%M,pause"

```