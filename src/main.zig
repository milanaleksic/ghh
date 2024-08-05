const std = @import("std");
const config = @import("config.zig");
const util = @import("util.zig");
const string = util.string;
const fatal = util.fatal;
const JiraService = @import("jira.zig").JiraService;
const args = @import("args.zig");

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    const gpaAlloc = gpa.allocator();

    var arena = std.heap.ArenaAllocator.init(gpaAlloc);
    defer arena.deinit();

    const allocator = arena.allocator();
    var env = try std.process.getEnvMap(allocator);
    defer env.deinit();

    util.setDebug(env);

    const cli_args = try args.CliArgs.parse(allocator);
    switch (cli_args) {
        .branch_from_issue => |value| {
            // TODO: figure out the default config path for the system
            const config_dir = try util.getDefaultConfigPath(allocator, env);
            if (config_dir == null) {
                fatal("No config directory found\n", .{});
            }
            const config_path = try std.fs.path.join(allocator, &[_][]const u8{ config_dir.?, "ghh", "config.toml" });

            var app_config = try config.parseConfig(allocator, config_path);
            defer app_config.deinit();

            if (app_config.match_repo(value.dir)) |repo| {
                if (repo.uses_jira) {
                    var jira = try JiraService.init(allocator, app_config.jira);
                    try jira.list_my_issues();
                } else {
                    fatal("Repo uses Github, not supported yet\n", .{});
                }
            } else {
                fatal("No repo config found in {s} for {s}\n", .{ config_path, value.dir });
            }
        },
        .help => {
            std.debug.print("Usage: ghh [command]\n", .{});
            std.debug.print("Commands:\n", .{});
            std.debug.print("  branch_from_issue [-d <project_dir>] - creates branch from an assigned issue\n", .{});
            std.debug.print("  help - shows this text\n", .{});
        },
    }
}
