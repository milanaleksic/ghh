const std = @import("std");

pub const string = []const u8;

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
