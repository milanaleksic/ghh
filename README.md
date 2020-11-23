# Github Helper (ghh)

Utility application to help me with day-to-day work

This utility is helping me to prepare remote standups:

- can list commits done in X days, sorted and grouped based on issue id
  + `ghh daily <no>` where `<no>` is number of days to list commits of
- delete old cards from github column
  + `ghh task-cleanup <column_id> <days>` where we delete all cards 
  from column `<column_id>` older than `<days>` days

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
user_token = 'YOUR_PERSONAL_GITHUB_TOKEN'

[[repo]]
location = '/Users/milan/SourceCode/docs'
author = 'My Name' # used to filter commits via `git --author xxx`

[[repo]]
location = '/Users/milan/SourceCode/terraform'
author = 'My Name' # used to filter commits via `git --author xxx`
```