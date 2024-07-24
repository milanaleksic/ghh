const std = @import("std");
const process = std.process;
const assert = std.debug.assert;
const config = @import("config.zig");
const util = @import("util.zig");
const fatal = util.fatal;
const string = util.string;
const JiraService = @import("jira.zig").JiraService;

// ref: https://github.com/tigerbeetle/tigerbeetle/blob/ae3ed332815c95cde149fd0559976f16facdd8cc/src/copyhound.zig
const CliArgs = union(enum) {
    branch_from_issue: struct { dir: string },
    help,

    fn parse(allocator: std.mem.Allocator) !CliArgs {
        var args = try std.process.argsWithAllocator(allocator);
        assert(args.skip());

        var subcommand: ?std.meta.Tag(CliArgs) = null;
        var dir: string = try std.fs.cwd().realpathAlloc(allocator, ".");

        while (args.next()) |raw_arg| {
            const arg = try allocator.dupe(u8, raw_arg);
            std.mem.replaceScalar(u8, arg, '-', '_');

            if (subcommand == null) {
                inline for (comptime std.enums.values(std.meta.Tag(CliArgs))) |tag| {
                    if (std.mem.eql(u8, arg, @tagName(tag))) {
                        subcommand = tag;
                        break;
                    }
                } else fatal("unknown subcommand: '{s}'", .{arg});

                continue;
            }

            if (subcommand != null and subcommand.? == .branch_from_issue and std.mem.eql(u8, arg, "_d")) {
                if (args.next()) |raw_dir| {
                    dir = raw_dir;
                }

                continue;
            }

            fatal("unexpected argument: {s}", .{arg});
        }

        if (subcommand == null) fatal("subcommand required", .{});
        return switch (subcommand.?) {
            .branch_from_issue => .{ .branch_from_issue = .{
                .dir = dir,
            } },
            .help => .help,
        };
    }
};

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    const gpaAlloc = gpa.allocator();

    var arena = std.heap.ArenaAllocator.init(gpaAlloc);
    defer arena.deinit();

    const allocator = arena.allocator();

    const cli_args = try CliArgs.parse(allocator);
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
