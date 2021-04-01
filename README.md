# Github Helper (ghh)

Utility application to help me with day-to-day work

```
ghh 0.0.0
Milan Aleksić <milan@aleksic.dev>

USAGE:
    ghh [FLAGS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -v, --verbose    A level of verbosity, and can be used multiple times
    -V, --version    Prints version information

SUBCOMMANDS:
    branch-from-issue    Propose branch name based on actively assigned project cards in "In
                         Progress" column
    daily                Give run-down of all things done in the commits during the previous
                         <days>
    help                 Prints this message or the help of the given subcommand(s)
    task-cleanup         Remove old project cards by archiving them
```

## Configuration

Put this config file in file `$config/ghh/config.toml`

`$config` depends on your platform:

|Platform | Value                                 | Example                                  |
| ------- | ------------------------------------- | ---------------------------------------- |
| Linux   | `$XDG_CONFIG_HOME` or `$HOME`/.config | /home/alice/.config                      |
| macOS   | `$HOME`/Library/Application Support   | /Users/Alice/Library/Application Support |
| Windows | `{FOLDERID_RoamingAppData}`           | C:\Users\Alice\AppData\Roaming           |

Each `repo` in the configuration must be a Github repo

```toml
user_name = 'milanaleksic'
user_token = 'YOUR_PERSONAL_GITHUB_TOKEN'

[[repo]]
# columnId used for branch-from-issue
in_progress_column = 1812763
location = '/Users/milan/SourceCode/docs'
author = 'My Name' # anything that can be used to filter commits via `git --author xxx`

[[repo]]
location = '/Users/milan/SourceCode/terraform'
author = 'My Name' # anything that can be used to filter commits via `git --author xxx`
```
