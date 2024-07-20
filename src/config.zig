const std = @import("std");
const tomlz = @import("tomlz");
const util = @import("util.zig");
const string = util.string;

const GithubConfig = struct {
    const Self = @This();

    username: string,
    token: string,

    fn deinit(self: *Self, allocator: std.mem.Allocator) void {
        allocator.free(self.username);
        allocator.free(self.token);
    }
};

const JiraConfig = struct {
    const Self = @This();

    username: string,
    token: string,
    url: string,

    fn deinit(self: *Self, allocator: std.mem.Allocator) void {
        allocator.free(self.username);
        allocator.free(self.token);
        allocator.free(self.url);
    }
};

const Config = struct {
    const Self = @This();

    github: GithubConfig,
    jira: JiraConfig,
    repos: std.ArrayList(Repo),
    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator, table: tomlz.Table) !Self {
        var repos = std.ArrayList(Repo).init(allocator);
        const reposRaw = table.getArray("repo").?.items();
        for (reposRaw) |repo| {
            try repos.append(Repo{
                .location = try allocator.dupe(u8, repo.table.getString("location").?),
                .author = try allocator.dupe(u8, repo.table.getString("author").?),
                .gh_in_progress_column = repo.table.getInteger("in_progress_column") orelse 0,
                .uses_jira = repo.table.getBool("uses_jira") orelse false,
            });
        }

        // TODO: handle missing keys
        // TODO: improve the config without subnamespaces
        return Self{
            .repos = repos,
            .jira = JiraConfig{
                .username = try allocator.dupe(u8, table.getString("jira_username").?),
                .token = try allocator.dupe(u8, table.getString("jira_token").?),
                .url = try allocator.dupe(u8, table.getString("jira_url").?),
            },
            .github = GithubConfig{
                .username = try allocator.dupe(u8, table.getString("user_name").?),
                .token = try allocator.dupe(u8, table.getString("user_token").?),
            },
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *Self) void {
        for (self.repos.items) |*repo| {
            repo.deinit(self.allocator);
        }
        self.repos.deinit();
        self.jira.deinit(self.allocator);
        self.github.deinit(self.allocator);
    }
};

const Repo = struct {
    const Self = @This();

    gh_in_progress_column: i64,
    uses_jira: bool,
    location: string,
    author: string,

    fn deinit(self: *Repo, allocator: std.mem.Allocator) void {
        allocator.free(self.location);
        allocator.free(self.author);
    }
};

fn openFile(allocator: std.mem.Allocator, inputFile: string) !string {
    const file = try std.fs.cwd().openFile(inputFile, .{});
    defer file.close();

    var buf_reader = std.io.bufferedReader(file.reader());
    var buffer = try allocator.alloc(u8, 0); // Start with an empty buffer

    while (true) {
        var temp_buf: [4096]u8 = undefined; // Temporary buffer for reading
        const bytesRead = try buf_reader.read(&temp_buf);
        if (bytesRead == 0) break; // End of file reached

        // Append the read bytes to the main buffer
        buffer = try allocator.realloc(buffer, buffer.len + bytesRead);
        std.mem.copyForwards(u8, buffer[buffer.len - bytesRead ..], temp_buf[0..bytesRead]);
    }

    return buffer;
}

fn parserConfigString(allocator: std.mem.Allocator, input: string) !Config {
    // TODO: handle invalid TOML syntax
    var table = try tomlz.parse(allocator, input);
    defer table.deinit(allocator);

    return try Config.init(allocator, table);
}

pub fn parseConfig(allocator: std.mem.Allocator, inputFile: string) !Config {
    // TODO: handle missing config file
    const config_data = try openFile(allocator, inputFile);
    defer allocator.free(config_data);

    return try parserConfigString(allocator, config_data);
}

test "parses non-empty config" {
    var cf = try parserConfigString(std.testing.allocator,
        \\ user_name = "ghuser"
        \\ user_token = "ghp_***"
        \\ jira_username = "test@test.com"
        \\ jira_url = "https://repo.atlassian.net"
        \\ jira_token = "jira_***"
        \\ [[repo]]
        \\ uses_jira = true
        \\ in_progress_column = 1000
        \\ location = '/Users/xxx/projects/ghh'
        \\ author = 'User'
    );
    defer cf.deinit();

    try std.testing.expectEqualSlices(u8, cf.github.username, "ghuser");
    try std.testing.expectEqualSlices(u8, cf.github.token, "ghp_***");
    try std.testing.expectEqualSlices(u8, cf.jira.username, "test@test.com");
    try std.testing.expectEqualSlices(u8, cf.jira.url, "https://repo.atlassian.net");
    try std.testing.expectEqualSlices(u8, cf.jira.token, "jira_***");
    try std.testing.expectEqual(cf.repos.items.len, 1);
    try std.testing.expectEqual(cf.repos.items[0].gh_in_progress_column, 1000);
    try std.testing.expectEqual(cf.repos.items[0].uses_jira, true);
    try std.testing.expectEqualSlices(u8, cf.repos.items[0].location, "/Users/xxx/projects/ghh");
    try std.testing.expectEqualSlices(u8, cf.repos.items[0].author, "User");
}

test "parses non-empty config with optionals" {
    var cf = try parserConfigString(std.testing.allocator,
        \\ user_name = "ghuser"
        \\ user_token = "ghp_***"
        \\ jira_username = "test@test.com"
        \\ jira_url = "https://repo.atlassian.net"
        \\ jira_token = "jira_***"
        \\ [[repo]]
        \\ location = '/Users/xxx/projects/ghh'
        \\ author = 'User'
    );
    defer cf.deinit();

    try std.testing.expectEqualSlices(u8, cf.github.username, "ghuser");
    try std.testing.expectEqualSlices(u8, cf.github.token, "ghp_***");
    try std.testing.expectEqualSlices(u8, cf.jira.username, "test@test.com");
    try std.testing.expectEqualSlices(u8, cf.jira.url, "https://repo.atlassian.net");
    try std.testing.expectEqualSlices(u8, cf.jira.token, "jira_***");
    try std.testing.expectEqual(cf.repos.items.len, 1);
    try std.testing.expectEqual(cf.repos.items[0].gh_in_progress_column, 0);
    try std.testing.expectEqual(cf.repos.items[0].uses_jira, false);
    try std.testing.expectEqualSlices(u8, cf.repos.items[0].location, "/Users/xxx/projects/ghh");
    try std.testing.expectEqualSlices(u8, cf.repos.items[0].author, "Usera");
}