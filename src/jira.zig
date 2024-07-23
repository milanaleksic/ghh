const std = @import("std");
const config = @import("config.zig");
const http = std.http;
const util = @import("util.zig");
const string = util.string;

const JiraDTOSearchResult = struct {
    issues: []struct {
        key: string,
        fields: struct {
            summary: string,
        },
    },
};

pub const JiraService = struct {
    const Self = @This();

    cfg: config.JiraConfig,
    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator, cfg: config.JiraConfig) !Self {
        return Self{
            .allocator = allocator,
            .cfg = cfg,
        };
    }

    pub fn list_my_issues(self: *Self) !void {
        const server_url = try std.Uri.parse(self.cfg.url);
        const uri: std.Uri = .{
            .scheme = server_url.scheme,
            .user = .{ .percent_encoded = self.cfg.username },
            .password = .{ .raw = self.cfg.token },
            .host = server_url.host,
            .port = server_url.port,
            .path = .{ .raw = "/rest/api/2/search" },
            // TODO: how to encode this _correctly_
            .query = .{ .percent_encoded = "jql=status%3D%22In%20Progress%22%20AND%20assignee%3DcurrentUser%28%29" },
            .fragment = null,
        };

        std.debug.print("Requesting {/?} on Jira\n", .{uri});

        var client = http.Client{
            .allocator = self.allocator,
        };
        defer client.deinit();

        const buf = try self.allocator.alloc(u8, 1024 * 1024 * 4);
        var req = try client.open(.GET, uri, .{
            .server_header_buffer = buf,
        });
        defer req.deinit();
        req.headers.user_agent = .{ .override = "ghh" };

        try req.send();
        try req.finish();
        try req.wait();

        if (req.response.status != .ok) {
            var rdr = req.reader();
            const body = try rdr.readAllAlloc(self.allocator, 1024 * 4);
            defer self.allocator.free(body);
            std.debug.print("Request failed with status {d}, body: \n{s}\n", .{ req.response.status, body });
            return;
        }

        var rdr = req.reader();
        const body = try rdr.readAllAlloc(self.allocator, 1024 * 1024 * 4);
        defer self.allocator.free(body);

        const parsed = try std.json.parseFromSlice(JiraDTOSearchResult, self.allocator, body, .{ .ignore_unknown_fields = true });
        defer parsed.deinit();

        const issues = parsed.value.issues;
        std.debug.print("Got {d} owned issue\n", .{issues.len});
        for (issues) |issue| {
            std.debug.print("Found {s}: {s}\n", .{ issue.key, issue.fields.summary });
            const writer = std.io.getStdOut().writer();
            const stupified = try util.stupify(self.allocator, issue.fields.summary);
            defer self.allocator.free(stupified);
            try writer.print("{s}_{s}\n", .{ issue.key, stupified });
        }
    }
};
