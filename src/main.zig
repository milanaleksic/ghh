const std = @import("std");
const process = std.process;
const config = @import("config.zig");
const util = @import("util.zig");
const string = util.string;

pub fn main() !void {
    var args = process.args();
    _ = args.skip();

    // TODO: add a help command
    std.debug.print("GHH by milan@aleksic.dev\n", .{});

    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    const gpaAlloc = gpa.allocator();

    var arena = std.heap.ArenaAllocator.init(gpaAlloc);
    defer arena.deinit();

    const allocator = arena.allocator();

    // TODO: add a command to run branch from issue, don't run it by default

    // TODO: figure out the default config path for the system
    const config_path = "/Users/milan/Library/Application Support/ghh/config.toml";

    var app_config = try config.parseConfig(allocator, config_path);
    defer app_config.deinit();

    const path = work_dir(allocator);
    defer allocator.free(path);

    if (match_repo(app_config, path)) |repo| {
        std.debug.print("Repo details found: {any}\n", .{repo});
    } else {
        std.debug.print("No repo config found in {s} for {s}\n", .{config_path, path});
    }
}

fn work_dir(allocator: std.mem.Allocator) string {
    var args = process.args();
    _ = args.skip();
    // TODO: add an arg to specify the working path
    return args.next() orelse {
        const current_dir = std.fs.cwd();
        return current_dir.realpathAlloc(allocator, ".") catch "unknown";
    };
}

fn match_repo(app_config: config.Config, path: []const u8) ?config.Repo {
    for (app_config.repos.items) |repo| {
        if (std.mem.eql(u8, repo.location, path)) {
            return repo;
        }
    }
    return null;
}