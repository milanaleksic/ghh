const std = @import("std");
const assert = std.debug.assert;
const util = @import("util.zig");
const fatal = util.fatal;
const string = util.string;

// ref: https://github.com/tigerbeetle/tigerbeetle/blob/ae3ed332815c95cde149fd0559976f16facdd8cc/src/copyhound.zig
pub const CliArgs = union(enum) {
    branch_from_issue: struct { dir: string },
    help,

    pub fn parse(allocator: std.mem.Allocator) !CliArgs {
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