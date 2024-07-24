const std = @import("std");
const config = @import("config.zig");
const util = @import("util.zig");
const string = util.string;
const JiraService = @import("jira.zig").JiraService;
const args = @import("args.zig");

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    const gpaAlloc = gpa.allocator();

    var arena = std.heap.ArenaAllocator.init(gpaAlloc);
    defer arena.deinit();

    const allocator = arena.allocator();

    const cli_args = try args.CliArgs.parse(allocator);
    switch (cli_args) {
        .branch_from_issue => |value| {
            // TODO: figure out the default config path for the system
            const config_path = "/Users/milan/Library/Application Support/ghh/config.toml";

            var app_config = try config.parseConfig(allocator, config_path);
            defer app_config.deinit();

            if (app_config.match_repo(value.dir)) |repo| {
                if (repo.uses_jira) {
                    var jira = try JiraService.init(allocator, app_config.jira);
                    try jira.list_my_issues();
                } else {
                    std.debug.print("Repo uses Github\n", .{});
                }
            } else {
                std.debug.print("No repo config found in {s} for {s}\n", .{ config_path, value.dir });
            }
        },
        .help => {
            std.debug.print("Usage: ghh [command]\n", .{});
            std.debug.print("Commands:\n", .{});
            std.debug.print("  branch_from_issue [-d <project_dir>]\n", .{});
        },
    }
}
