load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")

_download_attrs = {
    "finalize-stub-aarch64-linux": {
        "name": "finalize_stub_aarch64_linux",
        "url": "https://github.com/malt3/runfiles-stub/releases/download/v0.1.20251213/finalize-stub-aarch64-linux",
        "sha256": "f359e88589167d1c4ff5b06fe60e18bcc1aa16a21d49f81056b46a2d059c9a64",
    },
    "finalize-stub-aarch64-macos": {
        "name": "finalize_stub_aarch64_macos",
        "url": "https://github.com/malt3/runfiles-stub/releases/download/v0.1.20251213/finalize-stub-aarch64-macos",
        "sha256": "279a6976fc02a3903d2f8ad82edd6b58e50f0bbea6dc87cc0bc72005d3802fee",
    },
    "finalize-stub-x86_64-linux": {
        "name": "finalize_stub_x86_64_linux",
        "url": "https://github.com/malt3/runfiles-stub/releases/download/v0.1.20251213/finalize-stub-x86_64-linux",
        "sha256": "dce220b1ba5a2b44e48070a00e2a260923078bfda1b66f6ee584573ba90562cc",
    },
    "finalize-stub-x86_64-macos": {
        "name": "finalize_stub_x86_64_macos",
        "url": "https://github.com/malt3/runfiles-stub/releases/download/v0.1.20251213/finalize-stub-x86_64-macos",
        "sha256": "538277b9a5ca20b27e89fcc0497761782c10624eeb6e7aab974d215c2c6b82fe",
    },
    "finalize-stub-x86_64-windows.exe": {
        "name": "finalize_stub_x86_64_windows",
        "url": "https://github.com/malt3/runfiles-stub/releases/download/v0.1.20251213/finalize-stub-x86_64-windows.exe",
        "sha256": "fc8d023ec5dba3e058ad5354b7116b1c586bae22c17638c9a73282ec7b35e900",
    },
    "runfiles-stub-aarch64-linux": {
        "name": "runfiles_stub_aarch64_linux",
        "url": "https://github.com/malt3/runfiles-stub/releases/download/v0.1.20251213/runfiles-stub-aarch64-linux",
        "sha256": "b454ce9e990be145e22a3888efc82bfc6a6c1fb15e01703c75221bc5f878aada",
    },
    "runfiles-stub-aarch64-macos": {
        "name": "runfiles_stub_aarch64_macos",
        "url": "https://github.com/malt3/runfiles-stub/releases/download/v0.1.20251213/runfiles-stub-aarch64-macos",
        "sha256": "6e1234e7c3d888dd6e4d9e51be19216b87b5f70ebdc622eb0d8af856701c13d3",
    },
    "runfiles-stub-x86_64-linux": {
        "name": "runfiles_stub_x86_64_linux",
        "url": "https://github.com/malt3/runfiles-stub/releases/download/v0.1.20251213/runfiles-stub-x86_64-linux",
        "sha256": "a54004c133e44cd9eaff833deec4722a9249f42f52c2b2083ed8c3b44ab99e1d",
    },
    "runfiles-stub-x86_64-macos": {
        "name": "runfiles_stub_x86_64_macos",
        "url": "https://github.com/malt3/runfiles-stub/releases/download/v0.1.20251213/runfiles-stub-x86_64-macos",
        "sha256": "19fa6590570e7c0d1881b9ff5723b9279bd394a4818b933368c35646e46500ad",
    },
    "runfiles-stub-x86_64-windows.exe": {
        "name": "runfiles_stub_x86_64_windows",
        "url": "https://github.com/malt3/runfiles-stub/releases/download/v0.1.20251213/runfiles-stub-x86_64-windows.exe",
        "sha256": "a93a867cc1db6725c1173d42d427fbaf771d7e2049d3a6967643a631fc393b7e",
    },
}

def _non_module_dependencies_impl(ctx):
    for filename, attrs in _download_attrs.items():
        http_file(
            name = attrs["name"],
            url = attrs["url"],
            sha256 = attrs["sha256"],
            downloaded_file_path = filename,
            executable = True,
        )
    return ctx.extension_metadata(
        root_module_direct_deps = "all",
        root_module_direct_dev_deps = [],
        reproducible = True,
    )


non_module_dependencies = module_extension(
    implementation = _non_module_dependencies_impl,
)
