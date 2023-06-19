package(default_visibility = ["//visibility:public"])

load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "music_player_audio",
    srcs = [
      "src/decoder/mod.rs",
      "src/decoder/symphonia_decoder.rs",
      "src/fetch/mod.rs",
      "src/fetch/cache.rs",
      "src/fetch/client.rs",
      "src/fetch/receive.rs",
      "src/lib.rs",
      "src/range_set.rs",
    ],
    deps = [
      "//settings:music_player_settings",
      "@crate_index//:anyhow",
      "@crate_index//:bytes",
      "@crate_index//:futures-util",
      "@crate_index//:hyper",
      "@crate_index//:hyper-rustls",
      "@crate_index//:mime_guess",
      "@crate_index//:parking_lot",
      "@crate_index//:rustls",
      "@crate_index//:symphonia",
      "@crate_index//:tempfile",
      "@crate_index//:thiserror",
      "@crate_index//:tokio",
      "@crate_index//:url",
    ] + all_crate_deps(),
)