const std = @import("std");
const process = std.process;
const config = @import("config.zig");

pub fn main() !void {
    var args = process.args();
    _ = args.skip();
    std.debug.print("GHH by milan@aleksic.dev\n", .{});

    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    const gpaAlloc = gpa.allocator();

    var arena = std.heap.ArenaAllocator.init(gpaAlloc);
    defer arena.deinit();

    const allocator = arena.allocator();

    // TODO: figure out the default config path for the system
    var app_config = try config.parseConfig(allocator, "/Users/milan/Library/Application Support/ghh/config.toml");
    defer app_config.deinit();

    std.debug.print("value of jira_username is {s}\n", .{app_config.jira.username});
}
