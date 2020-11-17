# Github Helper (ghh)

Utility application to help me with day-to-day work

This utility is helping me to prepare remote standups:

- can list commits done in X days, sorted and grouped based on issue id
- more to come...

## Configuration

Put this config file in file `$config/ghh/config.toml`

`$config` depends on your platform:

|Platform | Value                                 | Example                                  |
| ------- | ------------------------------------- | ---------------------------------------- |
| Linux   | `$XDG_CONFIG_HOME` or `$HOME`/.config | /home/alice/.config                      |
| macOS   | `$HOME`/Library/Application Support   | /Users/Alice/Library/Application Support |
| Windows | `{FOLDERID_RoamingAppData}`           | C:\Users\Alice\AppData\Roaming           |

Each repo must be a Github repo

```toml
[[repo]]
location = '/Users/milan/SourceCode/docs'

[[repo]]
location = '/Users/milan/SourceCode/terraform'
```