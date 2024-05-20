const std = @import("std");
const process = std.process;
const tomlz = @import("tomlz");

pub fn main() !void {
    var args = process.args();
    _ = args.skip();
    std.debug.print("GHH by milan@aleksic.dev\n", .{});

    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    const gpaAlloc = gpa.allocator();

    var arena = std.heap.ArenaAllocator.init(gpaAlloc);
    defer arena.deinit();

    const allocator = arena.allocator();
    var table = try tomlz.parse(allocator,
        \\foo = 1
        \\bar = 2
    );
    defer table.deinit(allocator);

    std.debug.print("value of foo is {d}", .{table.getInteger("foo").?});
}

test "aaa" {
    try std.testing.expectEqual(54578, 54578);
}
