# Github Helper (ghh)

## This project is in early development stage

Utility application to help me with day-to-day work

```
❯ ghh help
Usage: ghh [command]
Commands:
  branch_from_issue [-d <project_dir>]
```

## Configuration

Put this config file in file `$config/ghh/config.toml`

`$config` depends on your platform:

| Platform | Value                                 | Example                                  |
| -------- | ------------------------------------- | ---------------------------------------- |
| Linux    | `$XDG_CONFIG_HOME` or `$HOME`/.config | /home/alice/.config                      |
| macOS    | `$HOME`/Library/Application Support   | /Users/Alice/Library/Application Support |
| Windows  | `{FOLDERID_RoamingAppData}`           | C:\Users\Alice\AppData\Roaming           |

Each `repo` in the configuration must be a Github repo

```toml
user_name = 'milanaleksic'
user_token = 'YOUR_PERSONAL_GITHUB_TOKEN'
# in case you are using JIRA for issue tracking instead of GH
jira_username = "my_jira_username"
jira_url = "https://cloud-jira.atlassian.net"
jira_token = "my_api_token"

[[repo]]
# columnId used for branch-from-issue
in_progress_column = 1812763
location = '/Users/milan/SourceCode/docs'
author = 'My Name' # anything that can be used to filter commits via `git --author xxx`

[[repo]]
# optional: if set then we will use JIRA for some supported features like branch-from-issue
uses_jira = true
location = '/Users/milan/SourceCode/terraform'
author = 'My Name' # anything that can be used to filter commits via `git --author xxx`
```
