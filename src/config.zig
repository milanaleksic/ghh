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

pub fn parseConfig(allocator: std.mem.Allocator, inputFile: string) !Config {
    // TODO: handle missing config file
    const config_data = try openFile(allocator, inputFile);
    defer allocator.free(config_data);

    // TODO: handle invalid TOML syntax
    var table = try tomlz.parse(allocator, config_data);
    defer table.deinit(allocator);

    return try Config.init(allocator, table);
}

test "aaa" {
    try std.testing.expectEqual(54578, 54578);
}