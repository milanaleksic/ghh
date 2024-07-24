const std = @import("std");

pub const string = []const u8;

/// Format and print an error message followed by the usage string to stderr,
/// then exit with an exit code of 1.
pub fn fatal(comptime fmt_string: []const u8, args: anytype) noreturn {
    const stderr = std.io.getStdErr().writer();
    stderr.print("error: " ++ fmt_string ++ "\n", args) catch {};
    std.process.exit(1);
}

var debug = false;

pub fn setDebug(env: std.process.EnvMap) void {
    if (env.get("DEBUG")) |dbg| {
        if (std.mem.eql(u8, dbg, "1")) {
            debug = true;
        }
    }
}

/// Format and print a debug message followed by the usage string to stderr,
/// if the `DEBUG` environment variable is set to `1`.
pub fn log_debug(comptime fmt_string: []const u8, args: anytype) void {
    if (debug) {
        const stderr = std.io.getStdErr().writer();
        stderr.print("debug: " ++ fmt_string ++ "\n", args) catch {};
    }
}

pub fn stupify(allocator: std.mem.Allocator, s: string) !string {
    var result = std.ArrayList(u8).init(allocator);
    defer result.deinit();
    var prev_fixed = true;
    for (s) |c| {
        if (std.ascii.isAlphanumeric(c)) {
            try result.append(std.ascii.toLower(c));
            prev_fixed = false;
        } else if (!prev_fixed) {
            try result.append('_');
            prev_fixed = true;
        }
    }
    return allocator.dupe(u8, result.items);
}

test "stupification" {
    const conv = try stupify(std.testing.allocator, "🧑‍🔬 [SPIKE|IDEA] Do something extraordinary");
    defer std.testing.allocator.free(conv);
    try std.testing.expectEqualSlices(
        u8,
        "spike_idea_do_something_extraordinary",
        conv
    );
}
